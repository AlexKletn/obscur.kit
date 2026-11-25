//! Diagnostic sink port.

use crate::diagnostic::Diagnostic;

/// A destination for structured diagnostics. Adapters may mirror these into
/// `tracing`, `DevTools` or test buffers (spec §7.8).
pub trait DiagnosticSink {
    /// Records a diagnostic.
    fn emit(&self, diagnostic: &Diagnostic);
}

#[cfg(test)]
mod tests {
    use super::super::NullDiagnosticSink;
    use super::DiagnosticSink;
    use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};

    #[test]
    fn null_sink_is_object_safe_and_swallows() {
        let sink: Box<dyn DiagnosticSink> = Box::new(NullDiagnosticSink);
        sink.emit(&Diagnostic::new(
            DiagnosticCode::new("test.null"),
            DiagnosticSeverity::Error,
            DiagnosticSubsystem::View,
            "dropped",
        ));
    }
}
