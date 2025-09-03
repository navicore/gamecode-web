use anyhow;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing;

#[derive(Debug)]
pub enum AppError {
    Internal(anyhow::Error),
    Unauthorized,
    BadRequest(String),
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Internal(err) => {
                tracing::error!("Internal error: {:?}", err);
                // Check if this is a model-specific error we should show to the user
                let error_str = err.to_string();
                if error_str.contains("Model error:") || error_str.contains("currentDate") {
                    (StatusCode::INTERNAL_SERVER_ERROR, error_str)
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
                }
            }
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
            }
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, msg)
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err)
    }
}