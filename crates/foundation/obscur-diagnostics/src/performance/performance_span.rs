#![doc = include_str!("performance_span.doc/_module.md")]

use crate::diagnostic::DiagnosticSubsystem;
use crate::performance::SpanAttribution;

#[doc = include_str!("performance_span.doc/PerformanceSpan.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerformanceSpan {
    #[doc = include_str!("performance_span.doc/name.md")]
    pub name: &'static str,
    #[doc = include_str!("performance_span.doc/subsystem.md")]
    pub subsystem: DiagnosticSubsystem,
    #[doc = include_str!("performance_span.doc/attribution.md")]
    pub attribution: SpanAttribution,
}

impl PerformanceSpan {
    #[doc = include_str!("performance_span.doc/new.md")]
    #[must_use]
    pub const fn new(name: &'static str, subsystem: DiagnosticSubsystem) -> Self {
        Self {
            name,
            subsystem,
            attribution: SpanAttribution::none(),
        }
    }

    #[doc = include_str!("performance_span.doc/with_attribution.md")]
    #[must_use]
    pub const fn with_attribution(mut self, attribution: SpanAttribution) -> Self {
        self.attribution = attribution;
        self
    }
}
