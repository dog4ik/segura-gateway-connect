use axum::{Json, extract::State, routing::post};
use reqwest::StatusCode;
use tracing::instrument;

use crate::{
    connect,
    gateway::{self, mask},
    state::AppState,
};

#[instrument(skip_all)]
async fn callback_handler(
    state: State<AppState>,
    axum::Json(callback): Json<serde_json::Value>,
) -> StatusCode {
    tracing::trace!(
        data = %mask::secure_value(&callback),
        "Received callback from external gateway"
    );
    let Ok(callback) = serde_json::from_value::<gateway::callback::CallbackPayload>(callback)
    else {
        tracing::warn!("Failed to deserialize callback body");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let mapping = match state.db.get_mapping(&callback.order_reference).await {
        Ok(Some(mapping)) => mapping,
        Ok(None) => {
            tracing::warn!("Gateway id mapping is not found in database");
            return StatusCode::NOT_FOUND;
        }
        Err(e) => {
            tracing::error!("Failed to retrieve mapping from the database: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    let status = match callback.payment_status {
        gateway::SeguraStatus::Pending => {
            tracing::warn!("Unexpected pending status in callback");
            return StatusCode::OK;
        }
        gateway::SeguraStatus::Success => connect::callback::CallbackStatus::Approved,
        gateway::SeguraStatus::Failed => connect::callback::CallbackStatus::Declined {
            reason: callback.status_description,
        },
    };

    let args = connect::callback::SendArguments {
        merchant_key: mapping.merchant_private_key,
        token: mapping.token,
        currency: callback.currency,
        status,
        amount: callback.amount,
    };

    match connect::callback::send_callback(args).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("Failed to send callback to gateway.connect: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub fn router() -> axum::Router<crate::state::AppState> {
    axum::Router::new().route("/callback", post(callback_handler))
}
