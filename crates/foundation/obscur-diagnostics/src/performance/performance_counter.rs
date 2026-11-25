//! A named monotonic performance counter.

use crate::diagnostic::DiagnosticSubsystem;

#[doc = include_str!("../../docs/api/performance/performance_counter.doc/PerformanceCounter.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerformanceCounter {
    /// Subsystem the counter is attributed to.
    subsystem: DiagnosticSubsystem,
    /// Stable, human-readable counter label.
    label: Box<str>,
    /// Current accumulated value.
    value: u64,
}

impl PerformanceCounter {
    /// Creates a counter for `subsystem` named `label`, starting at zero.
    #[must_use]
    pub fn new(subsystem: DiagnosticSubsystem, label: impl Into<Box<str>>) -> Self {
        Self {
            subsystem,
            label: label.into(),
            value: 0,
        }
    }

    /// Increments the counter by one (saturating at [`u64::MAX`]).
    pub fn increment(&mut self) {
        self.value = self.value.saturating_add(1);
    }

    /// Adds `n` to the counter (saturating at [`u64::MAX`]).
    pub fn add(&mut self, n: u64) {
        self.value = self.value.saturating_add(n);
    }

    /// Resets the counter back to zero.
    pub fn reset(&mut self) {
        self.value = 0;
    }

    /// Returns the current value.
    #[must_use]
    pub fn value(&self) -> u64 {
        self.value
    }

    /// Returns the subsystem this counter is attributed to.
    #[must_use]
    pub fn subsystem(&self) -> DiagnosticSubsystem {
        self.subsystem
    }

    /// Returns the counter label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_zero() {
        let c = PerformanceCounter::new(DiagnosticSubsystem::Layout, "layout.passes");
        assert_eq!(c.value(), 0);
        assert_eq!(c.subsystem(), DiagnosticSubsystem::Layout);
        assert_eq!(c.label(), "layout.passes");
    }

    #[test]
    fn increment_add_reset() {
        let mut c = PerformanceCounter::new(DiagnosticSubsystem::Style, "css.matches");
        c.increment();
        c.increment();
        assert_eq!(c.value(), 2);
        c.add(40);
        assert_eq!(c.value(), 42);
        c.reset();
        assert_eq!(c.value(), 0);
    }

    #[test]
    fn increment_saturates() {
        let mut c = PerformanceCounter::new(DiagnosticSubsystem::Runtime, "dispatch");
        c.add(u64::MAX);
        c.increment();
        assert_eq!(c.value(), u64::MAX);
        c.add(10);
        assert_eq!(c.value(), u64::MAX);
    }

    #[test]
    fn clone_eq() {
        let a = PerformanceCounter::new(DiagnosticSubsystem::Text, "shaping");
        let b = a.clone();
        assert_eq!(a, b);
    }
}
