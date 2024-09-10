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
    #[error("JWT error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        let status = match self {
            AppError::JwtError(_) => axum::http::StatusCode::FORBIDDEN,
            AppError::IOError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}
