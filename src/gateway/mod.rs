use std::fmt::Display;

use axum::http::HeaderMap;

use crate::{
    connect::{self, interaction_log::InteractionSpan},
    db::Db,
    gateway::{
        auth::authenticated_headers,
        error::{ErrorResponse, GatewayError},
    },
};

pub mod api;
mod auth;
/// External gateway callback payload
mod callback;
mod error;
/// Type conversions between external gateway and gateway.connect
mod from;
/// Requisite masking
pub mod mask;
mod payin;
/// External gateway status response
mod status;

pub type Result<T> = std::result::Result<T, GatewayError>;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SeguraOkResponse<T> {
    #[serde(rename = "requestTime")]
    pub request_time: String,
    pub status: bool,
    pub code: u16,
    pub message: String,
    pub data: T,
}

#[derive(Debug, serde::Deserialize)]
pub enum SeguraResponse<T> {
    #[serde(untagged)]
    Ok(SeguraOkResponse<T>),
    #[serde(untagged)]
    Err(ErrorResponse),
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SeguraStatus {
    Failed,
    #[default]
    Pending,
    Success,
}

impl<T> SeguraResponse<T> {
    pub fn into_std_result(self) -> std::result::Result<SeguraOkResponse<T>, ErrorResponse> {
        match self {
            SeguraResponse::Ok(ok) => Ok(ok),
            SeguraResponse::Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SeguraGateway {
    client: reqwest::Client,
}

#[derive(Debug, Clone, Copy)]
pub enum InitRequestUrlSuffix {
    Initialize,
    HostedPayment,
}

impl Display for InitRequestUrlSuffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initialize => f.write_str("initialize"),
            Self::HostedPayment => f.write_str("hosted-payment"),
        }
    }
}

impl SeguraGateway {
    #[cfg(debug_assertions)]
    const BASE_URL: &str = "https://ap-dev.segura-pay.com/api/v1/payment-gateway";
    #[cfg(not(debug_assertions))]
    const BASE_URL: &str = "https://api.segura-pay.com/api/v1/payment-gateway";

    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Self { client }
    }

    async fn init(
        &self,
        auth_headers: HeaderMap,
        request: payin::PaymentInitRequest<'_>,
        span: &mut InteractionSpan,
        url_suffix: InitRequestUrlSuffix,
    ) -> Result<SeguraOkResponse<payin::PaymentInitData>> {
        let secured_request = mask::secure_serializable(&request);
        let url = format!("{}/{}", Self::BASE_URL, url_suffix);
        tracing::debug!(%url, data = %secured_request, "Gateway API payment init request");
        span.set_request(url.clone(), &secured_request);
        let res = self
            .client
            .post(&url)
            .json(&request)
            .headers(auth_headers)
            .send()
            .await?;
        span.set_response_status(res.status().as_u16());

        let response = res.json::<serde_json::Value>().await?;
        let secured_response = mask::secure_value(&response);
        span.set_response(&secured_response);
        tracing::debug!(
            response = %secured_response,
            "Gateway API payment init response"
        );
        let res: SeguraResponse<_> = serde_json::from_value(response)?;
        Ok(res.into_std_result()?)
    }

    pub async fn init_h2h_payment(
        &self,
        pay_request: &connect::api::payment::GwConnectH2HPaymentRequest,
        span: &mut InteractionSpan,
    ) -> Result<SeguraOkResponse<payin::PaymentInitData>> {
        let client_id = &pay_request.settings.client_id;
        let secret = &pay_request.settings.secret;

        let headers = authenticated_headers(client_id, secret);
        let init_response = self
            .init(
                headers,
                pay_request.into(),
                span,
                InitRequestUrlSuffix::Initialize,
            )
            .await?;

        Ok(init_response)
    }

    pub async fn process_h2h_payment(
        &self,
        pay_request: &connect::api::payment::GwConnectH2HPaymentRequest,
        card_params: &connect::api::payment::H2HCardParams,
        db: Db,
        reference: &str,
        span: &mut InteractionSpan,
    ) -> Result<SeguraOkResponse<payin::PaymentProcessData>> {
        let request = payin::ProcessRequest::from(pay_request, card_params, reference);
        let secured_request = mask::secure_serializable(&request);
        let url = format!("{}/process", Self::BASE_URL);
        span.set_request(url.clone(), &secured_request);

        tracing::debug!(data = %secured_request, "Gateway API payment request");

        let mapping_insert = db.insert_mapping(
            &pay_request.payment.merchant_private_key,
            &pay_request.payment.token,
            reference,
        );

        let headers = authenticated_headers(
            &pay_request.settings.client_id,
            &pay_request.settings.secret,
        );

        let process_request = async {
            let res = self
                .client
                .post(url)
                .json(&request)
                .headers(headers)
                .send()
                .await?;
            span.set_response_status(res.status().as_u16());
            res.json::<serde_json::Value>().await
        };

        let (mapping_insert, response) = tokio::join!(mapping_insert, process_request);
        if let Err(e) = mapping_insert {
            tracing::error!("Failed to insert gateway id mapping: {e}");
        };

        let response = response?;
        let secured_response = mask::secure_value(&response);
        span.set_response(&secured_response);
        tracing::debug!(data = %secured_response, "Gateway API payment response");
        let res = serde_json::from_value::<SeguraResponse<_>>(response)?;
        Ok(res.into_std_result()?)
    }

    pub async fn hosted_payment(
        &self,
        pay_request: connect::api::payment::GwConnectH2HPaymentRequest,
        span: &mut InteractionSpan,
    ) -> Result<SeguraOkResponse<payin::PaymentInitData>> {
        let client_id = &pay_request.settings.client_id;
        let secret = &pay_request.settings.secret;
        let init_response = self
            .init(
                authenticated_headers(client_id, secret),
                (&pay_request).into(),
                span,
                InitRequestUrlSuffix::HostedPayment,
            )
            .await?;

        Ok(init_response)
    }
}
