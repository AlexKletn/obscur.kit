//! Structured diagnostics: typed codes, severity, context and sinks.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod diagnostic;
mod performance;
mod port;
mod subscription;

pub use diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticContext, DiagnosticSeverity, DiagnosticSubsystem,
    RecoveryAction,
};
pub use performance::{PerformanceBudget, PerformanceSpan};
pub use port::DiagnosticSink;
pub use subscription::DiagnosticFilter;
