use crate::performance::{BudgetViolation, PerformanceCounter, PerformanceSpan};

use super::TelemetrySink;

/// A telemetry sink that discards everything it receives.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NullTelemetrySink;

impl TelemetrySink for NullTelemetrySink {
    fn record_span(&mut self, _span: &PerformanceSpan) {}
    fn record_counter(&mut self, _counter: &PerformanceCounter) {}
    fn record_violation(&mut self, _violation: &BudgetViolation) {}
}
