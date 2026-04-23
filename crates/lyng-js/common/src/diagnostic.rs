//! Diagnostic primitives for error and warning reporting.
//!
//! Diagnostics are the shared error currency across lexer, parser, sema,
//! and compiler. They carry a severity, a message, and a primary span.

use crate::Span;
use std::fmt;

/// Diagnostic severity level.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// A warning that does not prevent compilation.
    Warning,
    /// A syntax or semantic error that prevents execution.
    Error,
}

impl fmt::Debug for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Warning => f.write_str("warning"),
            Severity::Error => f.write_str("error"),
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// A single diagnostic: an error or warning tied to a source location.
#[derive(Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub span: Span,
    pub message: String,
}

impl Diagnostic {
    /// Creates a new error diagnostic.
    #[inline]
    pub fn error(span: Span, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            span,
            message: message.into(),
        }
    }

    /// Creates a new warning diagnostic.
    #[inline]
    pub fn warning(span: Span, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            span,
            message: message.into(),
        }
    }

    /// Returns true if this is an error-level diagnostic.
    #[inline]
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

impl fmt::Debug for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?}: {}", self.severity, self.span, self.message)
    }
}

/// An accumulator for diagnostics emitted during a compilation phase.
#[derive(Clone, Default)]
pub struct DiagnosticList {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a diagnostic.
    #[inline]
    pub fn push(&mut self, diag: Diagnostic) {
        self.diagnostics.push(diag);
    }

    /// Adds an error diagnostic.
    #[inline]
    pub fn error(&mut self, span: Span, message: impl Into<String>) {
        self.push(Diagnostic::error(span, message));
    }

    /// Adds a warning diagnostic.
    #[inline]
    pub fn warning(&mut self, span: Span, message: impl Into<String>) {
        self.push(Diagnostic::warning(span, message));
    }

    /// Returns true if any error-level diagnostics have been emitted.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }

    /// Returns a slice of all diagnostics.
    pub fn as_slice(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns the number of diagnostics.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Consumes and returns the inner diagnostics.
    pub fn into_inner(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    /// Drains all diagnostics from `other` into this list.
    pub fn extend(&mut self, other: &mut DiagnosticList) {
        self.diagnostics.append(&mut other.diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceId;

    fn test_span() -> Span {
        Span::from_offsets(SourceId::new(0), 0, 5)
    }

    #[test]
    fn error_diagnostic() {
        let d = Diagnostic::error(test_span(), "unexpected token");
        assert!(d.is_error());
        assert_eq!(d.severity, Severity::Error);
    }

    #[test]
    fn warning_diagnostic() {
        let d = Diagnostic::warning(test_span(), "unreachable code");
        assert!(!d.is_error());
    }

    #[test]
    fn diagnostic_list_tracks_errors() {
        let mut list = DiagnosticList::new();
        assert!(!list.has_errors());
        list.warning(test_span(), "warn");
        assert!(!list.has_errors());
        list.error(test_span(), "err");
        assert!(list.has_errors());
        assert_eq!(list.len(), 2);
    }
}
