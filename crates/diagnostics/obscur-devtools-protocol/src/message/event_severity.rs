/// Severity of a diagnostic event, mirroring the runtime's severity ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventSeverity {
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
