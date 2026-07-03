//! Diagnostic sink port.

use crate::diagnostic::Diagnostic;

/// A destination for structured diagnostics. Adapters may mirror these into
/// `tracing`, `DevTools` or test buffers (spec §7.8).
pub trait DiagnosticSink {
    /// Records a diagnostic.
    fn emit(&self, diagnostic: &Diagnostic);
}
