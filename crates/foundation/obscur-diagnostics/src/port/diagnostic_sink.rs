#![doc = include_str!("diagnostic_sink.doc/_module.md")]

use crate::diagnostic::Diagnostic;

#[doc = include_str!("diagnostic_sink.doc/DiagnosticSink.md")]
pub trait DiagnosticSink {
    #[doc = include_str!("diagnostic_sink.doc/emit.md")]
    fn emit(&self, diagnostic: &Diagnostic);
}

#[doc = include_str!("diagnostic_sink.doc/NullDiagnosticSink.md")]
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
