use std::error::Error;
use std::fmt;

/// Public CLI error category for the thin `lyng-js` embedding surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CliErrorKind {
    Usage,
    Io,
    Host,
    Lowering,
    Vm,
    Internal,
}

/// Public CLI error for argument parsing and embedding failures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliError {
    kind: CliErrorKind,
    message: String,
}

impl CliError {
    #[inline]
    pub fn usage(message: impl Into<String>) -> Self {
        Self {
            kind: CliErrorKind::Usage,
            message: message.into(),
        }
    }

    #[inline]
    pub fn io(error: impl fmt::Display) -> Self {
        Self {
            kind: CliErrorKind::Io,
            message: error.to_string(),
        }
    }

    #[inline]
    pub fn host(error: impl fmt::Display) -> Self {
        Self {
            kind: CliErrorKind::Host,
            message: error.to_string(),
        }
    }

    #[inline]
    pub fn lowering(message: impl Into<String>) -> Self {
        Self {
            kind: CliErrorKind::Lowering,
            message: message.into(),
        }
    }

    #[inline]
    pub fn vm(message: impl Into<String>) -> Self {
        Self {
            kind: CliErrorKind::Vm,
            message: message.into(),
        }
    }

    #[inline]
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            kind: CliErrorKind::Internal,
            message: message.into(),
        }
    }

    #[inline]
    pub const fn kind(&self) -> CliErrorKind {
        self.kind
    }

    #[inline]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for CliError {}

/// Result alias for CLI operations.
pub type CliResult<T> = Result<T, CliError>;
