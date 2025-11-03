#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallbackPayload {
    pub currency: String,
    pub amount: usize,
    pub order_reference: String,
    #[serde(default)]
    pub payment_status: super::SeguraStatus,
    pub status_description: String,
}
