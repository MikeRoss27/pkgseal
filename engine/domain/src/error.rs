use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl DomainError {
    pub fn validation(msg: impl Into<String>) -> Self {
        DomainError::Validation(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        DomainError::NotFound(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        DomainError::Internal(msg.into())
    }
}
