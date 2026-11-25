//! The typed diagnostic produced when a measurement exceeds a budget.

use std::time::Duration;

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};

/// The stable diagnostic code carried by a budget violation.
const BUDGET_VIOLATION_CODE: DiagnosticCode = DiagnosticCode::new("perf.budget_exceeded");

#[doc = include_str!("../../docs/api/performance/budget_violation.doc/BudgetViolation.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetViolation {
    /// Subsystem whose budget was exceeded.
    subsystem: DiagnosticSubsystem,
    /// The budgeted limit.
    budget: Duration,
    /// The measured duration that exceeded the limit.
    measured: Duration,
}

impl BudgetViolation {
    /// Creates a budget violation for `subsystem`, given the `budget` limit and
    /// the `measured` duration that exceeded it.
    #[must_use]
    pub fn new(subsystem: DiagnosticSubsystem, budget: Duration, measured: Duration) -> Self {
        Self {
            subsystem,
            budget,
            measured,
        }
    }

    /// Returns the subsystem whose budget was exceeded.
    #[must_use]
    pub fn subsystem(&self) -> DiagnosticSubsystem {
        self.subsystem
    }

    /// Returns the budgeted limit.
    #[must_use]
    pub fn budget(&self) -> Duration {
        self.budget
    }

    /// Returns the measured duration that exceeded the budget.
    #[must_use]
    pub fn measured(&self) -> Duration {
        self.measured
    }

    /// Returns how far over budget the measurement ran (saturating at zero if
    /// the measurement did not actually exceed the budget).
    #[must_use]
    pub fn overage(&self) -> Duration {
        self.measured.saturating_sub(self.budget)
    }

    #[doc = include_str!("../../docs/api/performance/budget_violation.doc/ratio.md")]
    #[must_use]
    pub fn ratio(&self) -> f64 {
        let budget = self.budget.as_secs_f64();
        if budget == 0.0 {
            if self.measured.is_zero() {
                0.0
            } else {
                f64::INFINITY
            }
        } else {
            self.measured.as_secs_f64() / budget
        }
    }

    /// Converts this violation into a warning-level [`Diagnostic`] so it flows
    /// through the normal diagnostics channel (spec §7.9).
    #[must_use]
    pub fn to_diagnostic(&self) -> Diagnostic {
        let message = format!(
            "performance budget exceeded: measured {:?} over budget {:?} (overage {:?})",
            self.measured,
            self.budget,
            self.overage()
        );
        Diagnostic::new(
            BUDGET_VIOLATION_CODE,
            DiagnosticSeverity::Warning,
            self.subsystem,
            message,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overage_and_ratio() {
        let v = BudgetViolation::new(
            DiagnosticSubsystem::Layout,
            Duration::from_millis(10),
            Duration::from_millis(15),
        );
        assert_eq!(v.overage(), Duration::from_millis(5));
        assert!((v.ratio() - 1.5).abs() < 1e-9);
        assert_eq!(v.subsystem(), DiagnosticSubsystem::Layout);
        assert_eq!(v.budget(), Duration::from_millis(10));
        assert_eq!(v.measured(), Duration::from_millis(15));
    }

    #[test]
    fn ratio_zero_budget() {
        let over = BudgetViolation::new(
            DiagnosticSubsystem::Text,
            Duration::ZERO,
            Duration::from_millis(1),
        );
        assert_eq!(over.ratio(), f64::INFINITY);
        let none = BudgetViolation::new(DiagnosticSubsystem::Text, Duration::ZERO, Duration::ZERO);
        assert_eq!(none.ratio(), 0.0);
    }

    #[test]
    fn to_diagnostic_severity_and_subsystem() {
        let v = BudgetViolation::new(
            DiagnosticSubsystem::Render,
            Duration::from_millis(8),
            Duration::from_millis(20),
        );
        let d = v.to_diagnostic();
        assert_eq!(d.severity, DiagnosticSeverity::Warning);
        assert_eq!(d.subsystem, DiagnosticSubsystem::Render);
        assert_eq!(d.code, BUDGET_VIOLATION_CODE);
        assert!(d.message.contains("budget"));
    }
}
