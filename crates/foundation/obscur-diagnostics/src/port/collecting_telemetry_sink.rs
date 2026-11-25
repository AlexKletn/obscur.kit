use crate::performance::{BudgetViolation, PerformanceCounter, PerformanceSpan};

use super::TelemetrySink;

/// A telemetry sink that retains everything it receives, for tests and
/// DevTools-style inspection.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CollectingTelemetrySink {
    /// Spans recorded, in order.
    pub spans: Vec<PerformanceSpan>,
    /// Counters recorded, in order.
    pub counters: Vec<PerformanceCounter>,
    /// Budget violations recorded, in order.
    pub violations: Vec<BudgetViolation>,
}

impl CollectingTelemetrySink {
    /// Creates an empty collecting sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all recorded telemetry.
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
