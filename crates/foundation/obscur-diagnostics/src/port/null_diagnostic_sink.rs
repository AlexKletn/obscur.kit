use crate::diagnostic::Diagnostic;

use super::DiagnosticSink;

#[doc = include_str!("../../docs/api/port/diagnostic_sink.doc/NullDiagnosticSink.md")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NullDiagnosticSink;

impl DiagnosticSink for NullDiagnosticSink {
    fn emit(&self, _diagnostic: &Diagnostic) {}
}
