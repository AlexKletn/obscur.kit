#![doc = include_str!("budget_violation.doc/_module.md")]

use std::time::Duration;

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};

#[doc = include_str!("budget_violation.doc/BUDGET_VIOLATION_CODE.md")]
const BUDGET_VIOLATION_CODE: DiagnosticCode = DiagnosticCode::new("perf.budget_exceeded");

#[doc = include_str!("budget_violation.doc/BudgetViolation.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetViolation {
    #[doc = include_str!("budget_violation.doc/subsystem.md")]
    subsystem: DiagnosticSubsystem,
    #[doc = include_str!("budget_violation.doc/budget.md")]
    budget: Duration,
    #[doc = include_str!("budget_violation.doc/measured.md")]
    measured: Duration,
}

impl BudgetViolation {
    #[doc = include_str!("budget_violation.doc/new.md")]
    #[must_use]
    pub fn new(subsystem: DiagnosticSubsystem, budget: Duration, measured: Duration) -> Self {
        Self {
            subsystem,
            budget,
            measured,
        }
    }

    #[doc = include_str!("budget_violation.doc/subsystem_2.md")]
    #[must_use]
    pub fn subsystem(&self) -> DiagnosticSubsystem {
        self.subsystem
    }

    #[doc = include_str!("budget_violation.doc/budget_2.md")]
    #[must_use]
    pub fn budget(&self) -> Duration {
        self.budget
    }

    #[doc = include_str!("budget_violation.doc/measured_2.md")]
    #[must_use]
    pub fn measured(&self) -> Duration {
        self.measured
    }

    #[doc = include_str!("budget_violation.doc/overage.md")]
    #[must_use]
    pub fn overage(&self) -> Duration {
        self.measured.saturating_sub(self.budget)
    }

    #[doc = include_str!("budget_violation.doc/ratio.md")]
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

    #[doc = include_str!("budget_violation.doc/to_diagnostic.md")]
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
