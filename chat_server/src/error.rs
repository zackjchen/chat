use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Argon2 Password hash error: {0}")]
    Argon2Error(#[from] argon2::password_hash::Error),
}
