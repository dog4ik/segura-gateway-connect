#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInitRequest<'a> {
    pub amount: String,
    pub currency: &'a str,
    pub email: Option<&'a str>,
    pub country: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_url: Option<&'a str>,
    pub phone_number: Option<&'a str>,
    pub customer_name: Option<&'a str>,
    pub customer_id: &'a str,
    pub client_reference: String,
    pub narration: Option<&'a str>,
    pub address: Option<&'a str>,
    pub payment_method: &'a str,
    pub city: Option<&'a str>,
    pub state: Option<&'a str>,
    pub zip_code: Option<&'a str>,
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
    #[serde(rename = "customerdob")]
    pub customer_dob: Option<&'a str>, // Format: YYYY-MM-DD
    #[serde(rename = "cardholdername")]
    pub cardholder_name: &'a str,
    #[serde(rename = "customerfirstname")]
    pub customer_first_name: Option<&'a str>,
    #[serde(rename = "customerlastname")]
    pub customer_last_name: Option<&'a str>,
    pub card_scheme: Option<&'a str>, // e.g. "VISA"
    pub card_type: Option<&'a str>,   // e.g. "DEBIT"
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum PaymentProcessData {
    /// Standard payment response
    Standard(StandardPaymentData),
    /// 3DS card payment response(with redirect)
    ThreeDS(ThreeDSPaymentData),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]

#[serde(rename_all = "camelCase")]
pub struct StandardPaymentData {
    pub success: bool,
    pub currency: String,
    pub amount: f64,
    pub order_reference: String,
    pub status: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ThreeDSPaymentData {
    pub order_id: String,
    pub transaction_id: String,
    pub currency: String,
    pub amount: f64,
    pub status: super::SeguraStatus,
    pub created: String,
    pub descriptor: String,
    pub redirect: RedirectData,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RedirectData {
    pub url: String,
    pub method: String,
    pub target: String,
}
