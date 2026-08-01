#![doc = include_str!("telemetry_sink.doc/_module.md")]

use crate::performance::{BudgetViolation, PerformanceCounter, PerformanceSpan};

#[doc = include_str!("telemetry_sink.doc/TelemetrySink.md")]
pub trait TelemetrySink {
    #[doc = include_str!("telemetry_sink.doc/record_span.md")]
    fn record_span(&mut self, span: &PerformanceSpan);

    #[doc = include_str!("telemetry_sink.doc/record_counter.md")]
    fn record_counter(&mut self, counter: &PerformanceCounter);

    #[doc = include_str!("telemetry_sink.doc/record_violation.md")]
    fn record_violation(&mut self, violation: &BudgetViolation);
}

#[doc = include_str!("telemetry_sink.doc/NullTelemetrySink.md")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NullTelemetrySink;

impl TelemetrySink for NullTelemetrySink {
    fn record_span(&mut self, _span: &PerformanceSpan) {}
    fn record_counter(&mut self, _counter: &PerformanceCounter) {}
    fn record_violation(&mut self, _violation: &BudgetViolation) {}
}

#[doc = include_str!("telemetry_sink.doc/CollectingTelemetrySink.md")]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CollectingTelemetrySink {
    #[doc = include_str!("telemetry_sink.doc/spans.md")]
    pub spans: Vec<PerformanceSpan>,
    #[doc = include_str!("telemetry_sink.doc/counters.md")]
    pub counters: Vec<PerformanceCounter>,
    #[doc = include_str!("telemetry_sink.doc/violations.md")]
    pub violations: Vec<BudgetViolation>,
}

impl CollectingTelemetrySink {
    #[doc = include_str!("telemetry_sink.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("telemetry_sink.doc/clear.md")]
    pub fn clear(&mut self) {
        self.spans.clear();
        self.counters.clear();
        self.violations.clear();
    }
}

impl TelemetrySink for CollectingTelemetrySink {
    fn record_span(&mut self, span: &PerformanceSpan) {
        self.spans.push(*span);
    }

    fn record_counter(&mut self, counter: &PerformanceCounter) {
        self.counters.push(counter.clone());
    }

    fn record_violation(&mut self, violation: &BudgetViolation) {
        self.violations.push(violation.clone());
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::diagnostic::DiagnosticSubsystem;

    #[test]
    fn object_safe() {
        let mut sink = CollectingTelemetrySink::new();
        let _: &mut dyn TelemetrySink = &mut sink;
        let mut null = NullTelemetrySink;
        let _: &mut dyn TelemetrySink = &mut null;
    }

    #[test]
    fn null_sink_discards() {
        let mut null = NullTelemetrySink;
        let span = PerformanceSpan::new("frame", DiagnosticSubsystem::Render);
        null.record_span(&span);
        // No observable state; simply must not panic.
    }

    #[test]
    fn collecting_records_all() {
        let mut sink = CollectingTelemetrySink::new();
        let span = PerformanceSpan::new("frame", DiagnosticSubsystem::Render);
        let counter = PerformanceCounter::new(DiagnosticSubsystem::Layout, "passes");
        let violation = BudgetViolation::new(
            DiagnosticSubsystem::Style,
            Duration::from_millis(5),
            Duration::from_millis(9),
        );

        sink.record_span(&span);
        sink.record_counter(&counter);
        sink.record_violation(&violation);

        assert_eq!(sink.spans, vec![span]);
        assert_eq!(sink.counters, vec![counter]);
        assert_eq!(sink.violations, vec![violation]);

        sink.clear();
        assert!(sink.spans.is_empty());
        assert!(sink.counters.is_empty());
        assert!(sink.violations.is_empty());
    }

    #[test]
    fn records_through_dyn() {
        let mut sink = CollectingTelemetrySink::new();
        {
            let dynamic: &mut dyn TelemetrySink = &mut sink;
            dynamic.record_span(&PerformanceSpan::new("x", DiagnosticSubsystem::Text));
        }
        assert_eq!(sink.spans.len(), 1);
    }
}
