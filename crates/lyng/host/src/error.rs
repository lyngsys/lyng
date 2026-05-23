use std::error::Error;
use std::fmt;

/// Broad category for host-boundary failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostErrorKind {
    Unsupported,
    NotFound,
    InvalidRequest,
    Rejected,
    Internal,
}

/// Cold-path host-boundary error value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostError {
    kind: HostErrorKind,
    operation: &'static str,
    message: String,
}

impl HostError {
    #[inline]
    pub fn new(kind: HostErrorKind, operation: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            operation,
            message: message.into(),
        }
    }

    #[inline]
    pub fn unsupported(operation: &'static str, message: impl Into<String>) -> Self {
        Self::new(HostErrorKind::Unsupported, operation, message)
    }

    #[inline]
    pub fn not_found(operation: &'static str, message: impl Into<String>) -> Self {
        Self::new(HostErrorKind::NotFound, operation, message)
    }

    #[inline]
    pub fn invalid_request(operation: &'static str, message: impl Into<String>) -> Self {
        Self::new(HostErrorKind::InvalidRequest, operation, message)
    }

    #[inline]
    pub fn rejected(operation: &'static str, message: impl Into<String>) -> Self {
        Self::new(HostErrorKind::Rejected, operation, message)
    }

    #[inline]
    pub fn internal(operation: &'static str, message: impl Into<String>) -> Self {
        Self::new(HostErrorKind::Internal, operation, message)
    }

    #[inline]
    pub const fn kind(&self) -> HostErrorKind {
        self.kind
    }

    #[inline]
    pub const fn operation(&self) -> &'static str {
        self.operation
    }

    #[inline]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} failed: {}", self.operation, self.message)
    }
}

impl Error for HostError {}

/// Convenience alias for host-boundary operations.
pub type HostResult<T> = Result<T, HostError>;
