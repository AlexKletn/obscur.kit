//! A named performance span.

use crate::diagnostic::DiagnosticSubsystem;
use crate::performance::SpanAttribution;

/// A named span describing a measured region of work, attributed to a
/// subsystem (spec §7.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerformanceSpan {
    /// Stable span name.
    pub name: &'static str,
    /// Subsystem the span is attributed to.
    pub subsystem: DiagnosticSubsystem,
    #[doc = include_str!("../../docs/api/performance/performance_span.doc/attribution.md")]
    pub attribution: SpanAttribution,
}

impl PerformanceSpan {
    /// Creates a performance span.
    #[must_use]
    pub const fn new(name: &'static str, subsystem: DiagnosticSubsystem) -> Self {
        Self {
            name,
            subsystem,
            attribution: SpanAttribution::none(),
        }
    }

    /// Returns this span attributed to `attribution` (fluent).
    #[must_use]
    pub const fn with_attribution(mut self, attribution: SpanAttribution) -> Self {
        self.attribution = attribution;
        self
    }
}
