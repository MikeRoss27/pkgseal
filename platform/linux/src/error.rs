use thiserror::Error;

/// Root error type for the Linux platform crate.
///
/// Every fallible public API returns `Result<_, PlatformError>` so that
/// callers can map it to an `ApiError` without matching on strings.
///
/// No variant carries a shell command, a raw path supplied by the frontend,
/// or any other untrusted data that could be logged verbatim without
/// sanitisation.
#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("invalid package name: {0}")]
    InvalidPackageName(String),

    #[error("invalid flatpak app id: {0}")]
    InvalidFlatpakAppId(String),

    #[error("invalid privileged request: {0}")]
    InvalidPrivilegedRequest(String),

    #[error("process error: {0}")]
    Process(String),

    #[error("process timed out after {timeout_ms}ms: {program}")]
    Timeout { program: String, timeout_ms: u64 },

    #[error("output truncated: {kind} exceeded {limit_bytes} bytes")]
    OutputTruncated { kind: String, limit_bytes: usize },

    #[error("polkit error: {0}")]
    Polkit(String),

    #[error("not authorized for action {action}")]
    NotAuthorized { action: String },

    #[error("filesystem error: {0}")]
    Filesystem(String),

    #[error("desktop entry error: {0}")]
    DesktopEntry(String),

    #[error("environment error: {0}")]
    Environment(String),

    #[error("binary not available: {0}")]
    BinaryNotAvailable(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<std::io::Error> for PlatformError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl PlatformError {
    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        Self::InvalidArgument(msg.into())
    }

    pub fn invalid_privileged_request(msg: impl Into<String>) -> Self {
        Self::InvalidPrivilegedRequest(msg.into())
    }

    pub fn process(msg: impl Into<String>) -> Self {
        Self::Process(msg.into())
    }

    pub fn polkit(msg: impl Into<String>) -> Self {
        Self::Polkit(msg.into())
    }

    pub fn filesystem(msg: impl Into<String>) -> Self {
        Self::Filesystem(msg.into())
    }

    pub fn desktop_entry(msg: impl Into<String>) -> Self {
        Self::DesktopEntry(msg.into())
    }

    pub fn environment(msg: impl Into<String>) -> Self {
        Self::Environment(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_kind() {
        let err = PlatformError::InvalidArgument("empty arg".to_string());
        assert!(err.to_string().contains("invalid argument"));
    }

    #[test]
    fn timeout_display() {
        let err = PlatformError::Timeout {
            program: "/usr/bin/pacman".to_string(),
            timeout_ms: 5000,
        };
        assert!(err.to_string().contains("timed out"));
    }

    #[test]
    fn from_io() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err: PlatformError = io.into();
        assert!(matches!(err, PlatformError::Io(_)));
    }
}
