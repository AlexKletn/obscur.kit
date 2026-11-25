//! Diagnostic severity levels.

/// Severity of a diagnostic (spec §1.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticSeverity {
    /// Fine-grained tracing detail.
    Trace,
    /// Informational message.
    Info,
    /// A recoverable concern worth surfacing.
    Warning,
    /// A recoverable error handled by fallback.
    Error,
    /// An internal invariant failure.
    Critical,
}
