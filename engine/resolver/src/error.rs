use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("no candidates provided")]
    NoCandidates,
    #[error("normalization failed: {0}")]
    Normalization(String),
    #[error("signal extraction failed: {0}")]
    SignalExtraction(String),
    #[error("grouping failed: {0}")]
    Grouping(String),
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type ResolverResult<T> = Result<T, ResolverError>;
