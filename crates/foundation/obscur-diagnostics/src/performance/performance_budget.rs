#![doc = include_str!("performance_budget.doc/_module.md")]

use std::time::Duration;

use crate::diagnostic::DiagnosticSubsystem;
use crate::performance::BudgetViolation;

#[doc = include_str!("performance_budget.doc/PerformanceBudget.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerformanceBudget {
    #[doc = include_str!("performance_budget.doc/limit.md")]
    pub limit: Duration,
}

impl PerformanceBudget {
    #[doc = include_str!("performance_budget.doc/new.md")]
    #[must_use]
    pub const fn new(limit: Duration) -> Self {
        Self { limit }
    }

    #[doc = include_str!("performance_budget.doc/is_exceeded.md")]
    #[must_use]
    pub fn is_exceeded(self, elapsed: Duration) -> bool {
        elapsed > self.limit
    }

    #[doc = include_str!("performance_budget.doc/check.md")]
    #[must_use]
    pub fn check(
        self,
        subsystem: DiagnosticSubsystem,
        measured: Duration,
    ) -> Option<BudgetViolation> {
        if self.is_exceeded(measured) {
            Some(BudgetViolation::new(subsystem, self.limit, measured))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_none_under_budget() {
        let b = PerformanceBudget::new(Duration::from_millis(10));
        assert!(
            b.check(DiagnosticSubsystem::Layout, Duration::from_millis(5))
                .is_none()
        );
        // Exactly at the limit is not exceeded.
        assert!(
            b.check(DiagnosticSubsystem::Layout, Duration::from_millis(10))
                .is_none()
        );
    }

    #[test]
    fn check_some_over_budget() {
        let b = PerformanceBudget::new(Duration::from_millis(10));
        let v = b
            .check(DiagnosticSubsystem::Layout, Duration::from_millis(12))
            .expect("over budget");
        assert_eq!(v.subsystem(), DiagnosticSubsystem::Layout);
        assert_eq!(v.budget(), Duration::from_millis(10));
        assert_eq!(v.measured(), Duration::from_millis(12));
        assert_eq!(v.overage(), Duration::from_millis(2));
    }
}
