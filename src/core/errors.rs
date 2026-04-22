use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token not provided")]
    MissingToken,

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("You don't have permission to do this")]
    Forbidden,

    // --- Resource Errors ---
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Resource already exists: {0}")]
    Conflict(String),

    // --- Validation Errors ---
    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    // --- Infrastructure Errors ---
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Internal server error")]
    InternalServerError,
}

#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Map each error to an appropriate HTTP status code
        let (status, error_type, message) = match &self {
            AppError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", self.to_string())
            }
            AppError::MissingToken => (StatusCode::UNAUTHORIZED, "MISSING_TOKEN", self.to_string()),
            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, "INVALID_TOKEN", self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND", self.to_string()),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT", self.to_string()),
            AppError::ValidationError(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_ERROR",
                self.to_string(),
            ),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", self.to_string()),
            AppError::DatabaseError(e) => {
                // Log the actual DB error but don't expose it to clients
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "An internal error occurred".to_string(),
                )
            }
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An internal error occurred".to_string(),
            ),
        };

        let body = Json(json!({
            "error": error_type,
            "message": message,
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
