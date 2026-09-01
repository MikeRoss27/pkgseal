use pkgseal_domain::DomainError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("source unavailable: {0}")]
    Unavailable(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("domain error: {0}")]
    Domain(#[from] DomainError),
    #[error("internal: {0}")]
    Internal(String),
    #[error("validation error: {0}")]
    Validation(String),
}

impl SourceError {
    pub fn unavailable(msg: impl Into<String>) -> Self {
        SourceError::Unavailable(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        SourceError::NotFound(msg.into())
    }

    pub fn parse(msg: impl Into<String>) -> Self {
        SourceError::Parse(msg.into())
    }

    pub fn network(msg: impl Into<String>) -> Self {
        SourceError::Network(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        SourceError::Internal(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        SourceError::Validation(msg.into())
    }
}

pub type SourceResult<T> = Result<T, SourceError>;
