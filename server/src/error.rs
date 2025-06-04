use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

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
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
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