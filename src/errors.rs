use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Internal Server Error: {0}")]
    Internal(String),
}

// Implement IntoResponse for AppError
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "success": false,
            "message": message
        }));

        (status, body).into_response()
    }
}

// Implement conversions for third-party errors to make handler code clean
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        match &err {
            sqlx::Error::Database(db_err) => {
                // If it's a unique constraint violation (SQLSTATE 23505)
                if db_err.code().as_deref() == Some("23505") {
                    if let Some(constraint) = db_err.constraint() {
                        if constraint.contains("email") {
                            return AppError::Conflict("Email already registered".to_string());
                        }
                        if constraint.contains("receipt_no") {
                            return AppError::Conflict("Receipt number already exists".to_string());
                        }
                        if constraint.contains("student_id") {
                            return AppError::Conflict("Student ID already exists".to_string());
                        }
                    }
                    return AppError::Conflict("A record with this unique value already exists".to_string());
                }
                AppError::Internal(db_err.message().to_string())
            }
            sqlx::Error::RowNotFound => AppError::NotFound("Record not found".to_string()),
            _ => AppError::Internal("Database error occurred".to_string()),
        }
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        tracing::error!("Bcrypt error: {:?}", err);
        AppError::Internal("Encryption error".to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        tracing::error!("JWT error: {:?}", err);
        AppError::Unauthorized("Invalid or expired token".to_string())
    }
}
