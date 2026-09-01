use thiserror::Error;

/// Typed errors for the transaction subsystem.
///
/// Every variant is factual and user-presentable without leaking secrets.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TransactionError {
    #[error("invalid transition: {from} -> {to}")]
    InvalidTransition { from: String, to: String },

    #[error("validation error: {0}")]
    Validation(String),

    #[error("plan not found: {0}")]
    NotFound(String),

    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    #[error("cancelled: {0}")]
    Cancelled(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl TransactionError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn invalid_transition(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::InvalidTransition {
            from: from.into(),
            to: to.into(),
        }
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn execution_failed(msg: impl Into<String>) -> Self {
        Self::ExecutionFailed(msg.into())
    }

    pub fn cancelled(msg: impl Into<String>) -> Self {
        Self::Cancelled(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_invalid_transition() {
        let err = TransactionError::invalid_transition("planned", "succeeded");
        assert!(err.to_string().contains("planned"));
        assert!(err.to_string().contains("succeeded"));
    }

    #[test]
    fn validation_helper() {
        let err = TransactionError::validation("empty operations");
        assert!(matches!(err, TransactionError::Validation(_)));
    }
}
