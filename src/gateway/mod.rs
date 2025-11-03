use std::fmt::Display;

use axum::http::HeaderMap;
use serde::{Serialize, de::DeserializeOwned};

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

#[derive(Debug, Clone, Copy)]
pub enum InitRequestUrlSuffix {
    Initialize,
    HostedPayment,
}

impl Display for InitRequestUrlSuffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initialize => f.write_str("/initialize"),
            Self::HostedPayment => f.write_str("/hosted-payment"),
        }
    }
}

#[derive(Debug)]
pub struct RequestContext {
    auth_headers: HeaderMap,
    base_url: &'static str,
    client: reqwest::Client,
}

impl RequestContext {
    pub fn new(settings: &connect::api::payment::Settings) -> Self {
        let base_url = match settings.sandbox.unwrap_or(false) {
            true => "https://api-dev.segura-pay.com/api/v1/payment-gateway",
            false => "https://api.segura-pay.com/api/v1/payment-gateway",
        };
        let client = reqwest::ClientBuilder::new()
            .default_headers(authenticated_headers(&settings.client_id, &settings.secret))
            .build()
            .unwrap();
        Self {
            auth_headers: authenticated_headers(&settings.client_id, &settings.secret),
            base_url,
            client,
        }
    }

    pub async fn post<R: Serialize, T: DeserializeOwned>(
        &self,
        suffix: &str,
        body: &R,
        span: &mut InteractionSpan,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, suffix);
        let secured_request = mask::secure_serializable(body);
        tracing::debug!(%url, data = %secured_request, "Gateway API request");
        span.set_request(url.clone(), &secured_request);
        let res = self
            .client
            .post(&url)
            .json(&body)
            .headers(self.auth_headers())
            .send()
            .await?;
        let status = res.status().as_u16();
        span.set_response_status(status);
        let response = res.json::<serde_json::Value>().await?;
        let secured_response = mask::secure_value(&response);
        span.set_response(&secured_response);
        tracing::debug!(
            %url,
            %status,
            response = %secured_response,
            "Gateway API response"
        );
        let res: T = serde_json::from_value(response)?;
        Ok(res)
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        suffix: &str,
        span: &mut InteractionSpan,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, suffix);
        tracing::debug!(%url, "Gateway API request");
        span.set_request(url.clone(), &serde_json::Value::Null);
        let res = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;
        let status = res.status().as_u16();
        span.set_response_status(status);
        let response = res.json::<serde_json::Value>().await?;
        let secured_response = mask::secure_value(&response);
        span.set_response(&secured_response);
        tracing::debug!(
            %url,
            %status,
            response = %secured_response,
            "Gateway API response"
        );
        let res: T = serde_json::from_value(response)?;
        Ok(res)
    }

    pub fn auth_headers(&self) -> HeaderMap {
        self.auth_headers.clone()
    }

    async fn init(
        &self,
        request: payin::PaymentInitRequest<'_>,
        span: &mut InteractionSpan,
        url_suffix: InitRequestUrlSuffix,
    ) -> Result<SeguraOkResponse<payin::PaymentInitData>> {
        let res: SeguraResponse<_> = self.post(&url_suffix.to_string(), &request, span).await?;
        Ok(res.into_std_result()?)
    }

    pub async fn init_h2h_payment(
        &self,
        pay_request: payin::PaymentInitRequest<'_>,
        span: &mut InteractionSpan,
    ) -> Result<SeguraOkResponse<payin::PaymentInitData>> {
        let init_response = self
            .init(pay_request, span, InitRequestUrlSuffix::Initialize)
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

        let mapping_insert = db.insert_mapping(
            &pay_request.payment.merchant_private_key,
            &pay_request.payment.token,
            reference,
        );

        let process_request = async {
            self.post::<_, SeguraResponse<_>>("/process", &request, span)
                .await
        };

        let (mapping_insert, response) = tokio::join!(mapping_insert, process_request);
        let response = response?;
        if let Err(e) = mapping_insert {
            tracing::error!("Failed to insert gateway id mapping: {e}");
        };

        Ok(response.into_std_result()?)
    }

    pub async fn hosted_payment(
        &self,
        pay_request: connect::api::payment::GwConnectH2HPaymentRequest,
        span: &mut InteractionSpan,
    ) -> Result<SeguraOkResponse<payin::PaymentInitData>> {
        let init_response = self
            .init(
                (&pay_request).into(),
                span,
                InitRequestUrlSuffix::HostedPayment,
            )
            .await?;

        Ok(init_response)
    }
}
