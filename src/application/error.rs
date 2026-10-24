use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("user not found")]
    NotFound,
    #[error("user already exists")]
    Conflict,
    #[error("invalid input: {message}")]
    Validation { message: String },
    #[error("internal service error")]
    Internal,
}
