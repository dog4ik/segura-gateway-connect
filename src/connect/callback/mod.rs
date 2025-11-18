use std::time::Duration;

use axum::http::HeaderMap;
use axum_extra::headers::HeaderMapExt;
use serde::Serialize;

pub mod jwt;

const RETRY_ATTEMPTS: usize = 3;
const RETRY_DELAY: Duration = Duration::from_secs(3);

#[derive(Debug)]
pub struct SendArguments {
    pub merchant_key: String,
    pub token: String,
    pub currency: String,
    pub status: CallbackStatus,
    pub amount: usize,
}

#[derive(Serialize)]
pub struct CallbackPayload {
    #[serde(flatten)]
    pub status: CallbackStatus,
    pub currency: String,
    pub amount: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase", tag = "status")]
pub enum CallbackStatus {
    Declined { reason: String },
    Approved,
}

pub async fn send_callback(
    SendArguments {
        merchant_key,
        token,
        currency,
        status,
        amount,
    }: SendArguments,
) -> anyhow::Result<()> {
    let sign_key = std::env::var("SIGN_KEY").expect("SIGN_KEY env is defined");
    assert_eq!(sign_key.len(), 32);
    let key: [u8; 32] = sign_key.as_bytes().try_into().expect("length is 32 bytes");

    let payload = CallbackPayload {
        status,
        currency,
        amount,
    };
    let jwt = jwt::create_jwt(&payload, &merchant_key, &key)?;
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.typed_insert(axum_extra::headers::Authorization::bearer(&jwt).unwrap());
    let base = std::env::var("BUSINESS_URL").unwrap_or_else(|_| {
        tracing::warn!("BUSINESS_URL is not defined, using default one");
        "http://business:4000".to_string()
    });

    for i in 0..RETRY_ATTEMPTS {
        match client
            .post(format!("{base}/callbacks/v2/gateway_callbacks/{token}"))
            .headers(headers.clone())
            .json(&payload)
            .send()
            .await
            .and_then(|e| e.error_for_status())
        {
            Ok(_) => return Ok(()),
            Err(e) => {
                tracing::error!(
                    attempt = i + 1,
                    "Failed to send callback to gateway connect: {e}"
                );
                if i + 1 < RETRY_ATTEMPTS {
                    tokio::time::sleep(RETRY_DELAY).await;
                }
            }
        }
    }
    Err(anyhow::anyhow!("max attempts exceeded"))
}
