use axum::{
    body::Body,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorOutput {
    pub error: String,
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Argon2 Password hash error: {0}")]
    Argon2Error(#[from] argon2::password_hash::Error),
    #[error("JWT error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("Parse to HeaderValue error: {0}")]
    HeaderValueError(#[from] axum::http::header::InvalidHeaderValue),
    #[error("Email already exists: {0}")]
    EmailAlreadyExists(String),
    #[error("Create chat error: {0}")]
    CreateChatError(String),
    #[error("Create message error: {0}")]
    CreateMessageError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Not implemented: {0}")]
    ChatFileError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        let status = match self {
            AppError::SqlxError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Argon2Error(_) => axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            AppError::JwtError(_) => axum::http::StatusCode::FORBIDDEN,
            AppError::HeaderValueError(_) => axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            AppError::EmailAlreadyExists(_) => axum::http::StatusCode::CONFLICT,
            AppError::CreateChatError(_) => axum::http::StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => axum::http::StatusCode::NOT_FOUND,
            AppError::IOError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Unauthorized => axum::http::StatusCode::UNAUTHORIZED,
            AppError::CreateMessageError(_) => axum::http::StatusCode::BAD_REQUEST,
            AppError::ChatFileError(_) => axum::http::StatusCode::BAD_REQUEST,
        };

        (status, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}
