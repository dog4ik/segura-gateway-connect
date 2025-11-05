use crate::gateway::SeguraStatus;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInitRequest<'a> {
    pub amount: String,
    pub currency: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_url: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_name: Option<&'a str>,
    pub customer_id: &'a str,
    pub client_reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub narration: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<&'a str>,
    pub payment_method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip_code: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<&'a str>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInitData {
    pub reference: String,
    pub amount: f64,
    pub currency: String,
    pub redirect_url: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessRequest<'a> {
    pub pan: &'a str,
    pub cvv: &'a str,
    pub expiry: &'a str,
    pub expiry_month: &'a str,
    pub expiry_year: &'a str,
    pub reference: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "customerdob")]
    pub customer_dob: Option<&'a str>, // Format: YYYY-MM-DD
    #[serde(rename = "cardholdername")]
    pub cardholder_name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "customerfirstname")]
    pub customer_first_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "customerlastname")]
    pub customer_last_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_scheme: Option<String>, // e.g. "VISA"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_type: Option<&'a str>, // e.g. "DEBIT"
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum PaymentProcessData {
    /// 3DS card payment response(with redirect)
    ThreeDS(ThreeDSPaymentData),
    /// Standard payment response
    Standard(StandardPaymentData),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardPaymentData {
    pub success: bool,
    pub order_reference: String,
    pub status: SeguraStatus,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ThreeDSPaymentData {
    pub status: SeguraStatus,
    pub redirect: RedirectData,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RedirectData {
    pub url: String,
    pub method: String,
    pub target: String,
}
