use serde::{Deserialize, Serialize};

/// Structured IPC error — remplace `Result<T, String>` pour permettre
/// au frontend de brancher `toAppError()` sur `code` / `recoverable`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    #[allow(dead_code)]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            recoverable: true,
            details: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    #[allow(dead_code)]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            code: "NOT_FOUND".into(),
            message: message.into(),
            recoverable: false,
            details: None,
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            code: "VALIDATION_ERROR".into(),
            message: message.into(),
            recoverable: false,
            details: None,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: "INTERNAL_ERROR".into(),
            message: message.into(),
            recoverable: true,
            details: None,
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

/// Permet `map_err(|e| e.to_string())` -> conversion auto via `?` avec `thiserror`.
impl From<String> for ApiError {
    fn from(s: String) -> Self {
        Self::internal(s)
    }
}

impl From<&str> for ApiError {
    fn from(s: &str) -> Self {
        Self::internal(s.to_owned())
    }
}

/// Helpers pour convertir les erreurs domaine/sources sans exposer de `unwrap`.
pub fn internal_err<E: std::fmt::Display>(e: E) -> ApiError {
    ApiError::internal(e.to_string())
}

/// Validation d'entrée côté command — correspond à `VALIDATION_ERROR` frontend.
pub fn validation_err<E: std::fmt::Display>(e: E) -> ApiError {
    ApiError::validation(e.to_string())
}
