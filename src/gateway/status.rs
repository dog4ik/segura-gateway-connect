use serde::{Deserialize, Serialize};

use crate::{
    connect::interaction_log::InteractionSpan,
    gateway::{SeguraGateway, auth, mask},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SeguraStatusResponse {
    pub status: bool,
    pub code: u16,
    pub message: String,
    pub data: SeguraStatusData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeguraStatusData {
    pub currency: String,
    pub amount: usize,
    #[serde(rename = "paymentReference")]
    pub payment_reference: String,
    #[serde(default)]
    pub status: super::SeguraStatus,
}

impl SeguraGateway {
    pub async fn status(
        &self,
        client_id: &str,
        secret: &str,
        reference: &str,
        span: &mut InteractionSpan,
    ) -> anyhow::Result<SeguraStatusResponse> {
        let headers = auth::authenticated_headers(client_id, secret);
        let url = format!("{}/status/{}", Self::BASE_URL, reference);
        span.set_request(url.clone(), &serde_json::Value::Null);
        tracing::debug!(%url, "Gateway API status request");
        let response = self.client.get(url).headers(headers).send().await?;
        let status = response.status();
        span.set_response_status(status.as_u16());
        let response = response.json::<serde_json::Value>().await?;
        let secured_response = mask::secure_value(&response);
        span.set_response(&secured_response);
        tracing::debug!(data = %secured_response, %status, "Gateway API status response");
        let result = serde_json::from_value(response)?;
        Ok(result)
    }
}
