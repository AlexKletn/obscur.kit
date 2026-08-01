#![doc = include_str!("lib.doc/_module.md")]
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
pub use performance::{
    BudgetViolation, PerformanceBudget, PerformanceCounter, PerformanceSpan, SpanAttribution,
};
pub use port::{
    CollectingTelemetrySink, DiagnosticSink, NullDiagnosticSink, NullTelemetrySink, TelemetrySink,
};
pub use subscription::{
    ChannelSink, DiagnosticBufferPolicy, DiagnosticChannel, DiagnosticFilter,
    DiagnosticSubscription, OverflowStrategy, SubscriptionId,
};
