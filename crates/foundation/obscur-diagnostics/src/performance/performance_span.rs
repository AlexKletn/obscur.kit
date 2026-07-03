//! A named performance span.

use crate::diagnostic::DiagnosticSubsystem;

/// A named span describing a measured region of work, attributed to a
/// subsystem (spec §7.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerformanceSpan {
    /// Stable span name.
    pub name: &'static str,
    /// Subsystem the span is attributed to.
    pub subsystem: DiagnosticSubsystem,
}

impl PerformanceSpan {
    /// Creates a performance span.
    #[must_use]
    pub const fn new(name: &'static str, subsystem: DiagnosticSubsystem) -> Self {
        Self { name, subsystem }
    }
}
