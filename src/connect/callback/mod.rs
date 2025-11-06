use axum::http::HeaderMap;
use axum_extra::headers::HeaderMapExt;
use serde::Serialize;

pub mod jwt;

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
    let base = option_env!("BUSINESS_URL").unwrap_or("https://business.paysure.global");
    client
        .post(format!("{base}/callbacks/v2/gateway_callbacks/{token}"))
        .headers(headers)
        .json(&payload)
        .send()
        .await?
        .text()
        .await?;
    Ok(())
}
