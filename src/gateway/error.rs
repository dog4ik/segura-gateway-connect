use std::fmt::Display;

use serde::de::Error;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse {
    #[serde(rename = "responseCode")]
    pub response_code: String,
    #[serde(rename = "responseMessage")]
    pub response_message: String,
    pub errors: Vec<ErrorDetail>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ErrorDetail {
    #[serde(rename = "fieldName")]
    pub field_name: String,
    pub message: String,
}

#[derive(Debug)]
pub enum GatewayError {
    RequestError(reqwest::Error),
    GatewayResponse(ErrorResponse),
    GatewayDeserialization(serde_json::Error),
}

impl From<reqwest::Error> for GatewayError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_decode() {
            return Self::GatewayDeserialization(serde_json::Error::custom(
                "failed to decode response body",
            ));
        }
        Self::RequestError(value)
    }
}

impl From<ErrorResponse> for GatewayError {
    fn from(value: ErrorResponse) -> Self {
        Self::GatewayResponse(value)
    }
}

impl From<serde_json::Error> for GatewayError {
    fn from(value: serde_json::Error) -> Self {
        Self::GatewayDeserialization(value)
    }
}

impl std::error::Error for GatewayError {}

impl Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GatewayError::RequestError(e) => write!(f, "http request error: {e}"),
            GatewayError::GatewayResponse(error_response) => {
                write!(f, "gateway response: {}", error_response.response_message)
            }
            GatewayError::GatewayDeserialization(e) => {
                write!(f, "gateway response deserialization: {e}")
            }
        }
    }
}
