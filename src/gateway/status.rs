use serde::{Deserialize, Serialize};

use crate::{
    connect::interaction_log::InteractionSpan,
    gateway::{self, RequestContext},
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

impl RequestContext {
    pub async fn status(
        &self,
        reference: &str,
        span: &mut InteractionSpan,
    ) -> gateway::Result<SeguraStatusResponse> {
        let url = format!("/status/{}", reference);
        self.get(&url, span).await
    }
}
