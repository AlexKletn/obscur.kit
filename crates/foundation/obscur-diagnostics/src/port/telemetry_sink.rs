//! Telemetry sink port.

use crate::performance::{BudgetViolation, PerformanceCounter, PerformanceSpan};

#[doc = include_str!("../../docs/api/port/telemetry_sink.doc/TelemetrySink.md")]
pub trait TelemetrySink {
    /// Records a completed performance span.
    fn record_span(&mut self, span: &PerformanceSpan);

    /// Records the current state of a performance counter.
    fn record_counter(&mut self, counter: &PerformanceCounter);

    /// Records a budget violation.
    fn record_violation(&mut self, violation: &BudgetViolation);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::super::{CollectingTelemetrySink, NullTelemetrySink};
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
