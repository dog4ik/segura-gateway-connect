use axum::{extract::State, response::IntoResponse, routing::post};
use serde::Serialize;
use tracing::instrument;

use crate::{
    connect::{
        GwConnectErrorResponse, Result,
        interaction_log::{InteractionLog, InteractionSpan},
        status,
    },
    gateway::SeguraGateway,
    state::AppState,
};

#[instrument(skip_all)]
pub async fn pay(
    State(AppState { gate, db, .. }): State<AppState>,
    Json(payment): Json<payment::GwConnectH2HPaymentRequest>,
) -> Result<GwConnectResponse<GwConnectH2HPaymentResponse>> {
    let mut span = InteractionSpan::enter();
    match &payment.params.card_params {
        Some(card_params) => match gate.init_h2h_payment(&payment, &mut span).await {
            Ok(init_response) => {
                let init_log = span.interaction_log("init_payment");
                let mut process_span = InteractionSpan::enter();
                match gate
                    .process_h2h_payment(
                        &payment,
                        card_params,
                        db,
                        &init_response.data.reference,
                        &mut process_span,
                    )
                    .await
                {
                    Ok(res) => {
                        let process_log = process_span.interaction_log("payment");
                        Ok(GwConnectResponse::<GwConnectH2HPaymentResponse>::new(
                            (res, init_response.data.reference).into(),
                            vec![init_log, process_log],
                        ))
                    }
                    Err(e) => {
                        let process_log = process_span.interaction_log("payment");
                        Err(GwConnectErrorResponse::new(
                            e.to_string(),
                            vec![init_log, process_log],
                        ))
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to init h2h payment: {e}");
                let log = span.interaction_log("payment");
                Err(GwConnectErrorResponse::new(e.to_string(), vec![log]))
            }
        },
        None => match gate.hosted_payment(payment, &mut span).await {
            Ok(res) => {
                span.set_response(&res);
                let log = span.interaction_log("payment");
                tracing::info!(code = res.code, "Created payment");
                Ok(GwConnectResponse::<GwConnectH2HPaymentResponse>::new(
                    res.into(),
                    vec![log],
                ))
            }
            Err(e) => {
                tracing::error!("Failed to create a payment: {e}");
                let log = span.interaction_log("payment");
                Err(GwConnectErrorResponse::new(e.to_string(), vec![log]))
            }
        },
    }
}

#[instrument(skip_all)]
pub async fn status(
    State(gate): State<SeguraGateway>,
    Json(status_request): Json<status::req::Request>,
) -> Result<GwConnectResponse<status::res::Status>> {
    let mut span = InteractionSpan::enter();
    tracing::debug!(token = %status_request.payment.token, gateway_token = %status_request.payment.gateway_token, "Connect API status request");
    match gate
        .status(
            &status_request.settings.client_id,
            &status_request.settings.secret,
            &status_request.payment.gateway_token,
            &mut span,
        )
        .await
    {
        Ok(status) => {
            let log = span.interaction_log("status");
            tracing::info!(id = %status_request.payment.token, "Dispatched transaction status");
            Ok(GwConnectResponse::<status::res::Status>::new(
                status.into(),
                vec![log],
            ))
        }
        Err(e) => {
            tracing::error!("Failed to fetch transaction status: {e}");
            let log = span.interaction_log("status");
            Err(GwConnectErrorResponse::new(e.to_string(), vec![log]))
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GwConnectResponse<T> {
    result: bool,
    logs: Vec<InteractionLog>,
    #[serde(flatten)]
    data: T,
}

impl<T> GwConnectResponse<T> {
    pub fn new(data: T, logs: Vec<InteractionLog>) -> Self {
        // Add error field if result is false, keep logs
        Self {
            result: true,
            logs,
            data,
        }
    }
}

impl<T: Serialize> IntoResponse for GwConnectResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let value = serde_json::to_value(self).unwrap();
        tracing::debug!(data = %crate::gateway::mask::secure_value(&value), "Connect API response payload");
        axum::Json(value).into_response()
    }
}

pub mod payment {

    use serde::Deserialize;

    #[derive(Debug, Deserialize, Clone)]
    pub struct GwConnectH2HPaymentRequest {
        pub processing_url: String,
        pub payment: Payment,
        pub params: H2HParams,
        pub settings: Settings,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct H2HParams {
        #[serde(flatten)]
        pub card_params: Option<H2HCardParams>,
        pub address: Option<String>,
        pub first_name: Option<String>,
        pub last_name: Option<String>,
        pub city: Option<String>,
        pub birthday: Option<String>,
        pub postcode: Option<String>,
        pub phone: Option<String>,
        pub email: Option<String>,
        pub country: Option<String>,
        pub state: Option<String>,
    }

    #[derive(Debug, Clone)]
    pub struct H2HCardParams {
        pub cvv: String,
        pub expires: String,
        pub pan: String,
        pub holder: String,
    }

    // Implement deserialize manually to conceal any "helpful" error messages that can leak
    // sensitive data
    impl<'de> serde::de::Deserialize<'de> for H2HCardParams {
        fn deserialize<D>(deserializer: D) -> Result<H2HCardParams, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            #[derive(Debug, Clone, Deserialize)]
            struct H2HCardParamsShadow {
                cvv: String,
                expires: String,
                pan: String,
                holder: String,
            }

            impl From<H2HCardParamsShadow> for H2HCardParams {
                fn from(
                    H2HCardParamsShadow {
                        cvv,
                        expires,
                        pan,
                        holder,
                    }: H2HCardParamsShadow,
                ) -> Self {
                    Self {
                        cvv,
                        expires,
                        pan,
                        holder,
                    }
                }
            }

            H2HCardParamsShadow::deserialize(deserializer)
                .map(Into::into)
                .map_err(|_| serde::de::Error::custom("failed to deserialize card data"))
        }
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct Payment {
        pub gateway_amount: usize,
        pub gateway_currency: String,
        pub product: String,
        pub ip: Option<String>,
        pub token: String,
        pub merchant_private_key: String,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct Settings {
        pub client_id: String,
        pub secret: String,
    }
}

#[derive(Debug, Serialize)]
pub struct GwConnectH2HPaymentResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_request: Option<RedirectRequest>,
    pub result: super::Status,
    pub gateway_token: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct RedirectRequest {
    pub url: String,
    #[serde(rename = "type")]
    pub kind: RedirectRequestType,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "snake_case")]
#[allow(unused)]
pub enum RedirectRequestType {
    PostIframes,
    #[default]
    GetWithProcessing,
    Get,
    Post,
    RedirectHtml,
}

pub fn router() -> axum::Router<crate::state::AppState> {
    axum::Router::new()
        .route("/pay", post(pay))
        .route("/status", post(status))
}

/// `Json` extractor wrapper that customizes the error from `axum::extract::Json`
pub struct Json<T>(pub T);

impl<S, T> axum::extract::FromRequest<S> for Json<T>
where
    T: serde::de::DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = axum::Json<GwConnectErrorResponse>;

    async fn from_request(
        req: axum::http::Request<axum::body::Body>,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let rejection = match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => return Ok(Self(value)),
            Err(e) => e.to_string(),
        };
        Err(axum::Json(GwConnectErrorResponse::new(rejection, vec![])))
    }
}
