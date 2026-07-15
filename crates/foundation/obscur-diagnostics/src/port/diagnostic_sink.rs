//! Diagnostic sink port.

use crate::diagnostic::Diagnostic;

/// A destination for structured diagnostics. Adapters may mirror these into
/// `tracing`, `DevTools` or test buffers (spec §7.8).
pub trait DiagnosticSink {
    /// Records a diagnostic.
    fn emit(&self, diagnostic: &Diagnostic);
}

/// A diagnostic sink that discards everything it receives.
///
/// The default destination for an engine that has not been given a real sink:
/// diagnostics are still *produced* — so a failure follows exactly the same code
/// path whether or not anyone is listening — and then dropped.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NullDiagnosticSink;

impl DiagnosticSink for NullDiagnosticSink {
    fn emit(&self, _diagnostic: &Diagnostic) {}
}

#[cfg(test)]
mod tests {
    use super::{DiagnosticSink, NullDiagnosticSink};
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
