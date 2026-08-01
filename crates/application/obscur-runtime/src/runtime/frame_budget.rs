#![doc = include_str!("frame_budget.doc/_module.md")]

use std::time::Duration;

use obscur_diagnostics::{BudgetViolation, DiagnosticSubsystem, PerformanceBudget};

#[doc = include_str!("frame_budget.doc/FrameBudget.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameBudget {
    #[doc = include_str!("frame_budget.doc/reconcile_nodes.md")]
    reconcile_nodes: Option<u64>,
    #[doc = include_str!("frame_budget.doc/style_nodes.md")]
    style_nodes: Option<u64>,
    #[doc = include_str!("frame_budget.doc/layout_boxes.md")]
    layout_boxes: Option<u64>,
    #[doc = include_str!("frame_budget.doc/compose_commands.md")]
    compose_commands: Option<u64>,
}

impl FrameBudget {
    #[doc = include_str!("frame_budget.doc/unbounded.md")]
    #[must_use]
    pub const fn unbounded() -> Self {
        Self {
            reconcile_nodes: None,
            style_nodes: None,
            layout_boxes: None,
            compose_commands: None,
        }
    }

    #[doc = include_str!("frame_budget.doc/with_reconcile_nodes.md")]
    #[must_use]
    pub const fn with_reconcile_nodes(mut self, max: u64) -> Self {
        self.reconcile_nodes = Some(max);
        self
    }

    #[doc = include_str!("frame_budget.doc/with_style_nodes.md")]
    #[must_use]
    pub const fn with_style_nodes(mut self, max: u64) -> Self {
        self.style_nodes = Some(max);
        self
    }

    #[doc = include_str!("frame_budget.doc/with_layout_boxes.md")]
    #[must_use]
    pub const fn with_layout_boxes(mut self, max: u64) -> Self {
        self.layout_boxes = Some(max);
        self
    }

    #[doc = include_str!("frame_budget.doc/with_compose_commands.md")]
    #[must_use]
    pub const fn with_compose_commands(mut self, max: u64) -> Self {
        self.compose_commands = Some(max);
        self
    }

    #[doc = include_str!("frame_budget.doc/check_reconcile.md")]
    #[must_use]
    pub(crate) fn check_reconcile(self, nodes: u64) -> Option<BudgetViolation> {
        Self::check(self.reconcile_nodes, DiagnosticSubsystem::Tree, nodes)
    }

    #[doc = include_str!("frame_budget.doc/check_style.md")]
    #[must_use]
    pub(crate) fn check_style(self, count: u64) -> Option<BudgetViolation> {
        Self::check(self.style_nodes, DiagnosticSubsystem::Style, count)
    }

    #[doc = include_str!("frame_budget.doc/check_layout.md")]
    #[must_use]
    pub(crate) fn check_layout(self, count: u64) -> Option<BudgetViolation> {
        Self::check(self.layout_boxes, DiagnosticSubsystem::Layout, count)
    }

    #[doc = include_str!("frame_budget.doc/check_compose.md")]
    #[must_use]
    pub(crate) fn check_compose(self, count: u64) -> Option<BudgetViolation> {
        Self::check(self.compose_commands, DiagnosticSubsystem::Render, count)
    }

    #[doc = include_str!("frame_budget.doc/check.md")]
    #[must_use]
    fn check(
        limit: Option<u64>,
        subsystem: DiagnosticSubsystem,
        measured: u64,
    ) -> Option<BudgetViolation> {
        let limit = limit?;
        PerformanceBudget::new(Duration::from_nanos(limit))
            .check(subsystem, Duration::from_nanos(measured))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbounded_never_violates() {
        let b = FrameBudget::unbounded();
        assert!(b.check_reconcile(1_000).is_none());
        assert!(b.check_style(1_000).is_none());
        assert!(b.check_layout(1_000).is_none());
        assert!(b.check_compose(1_000).is_none());
    }

    #[test]
    fn default_is_unbounded() {
        assert_eq!(FrameBudget::default(), FrameBudget::unbounded());
    }

    #[test]
    fn at_limit_is_not_exceeded() {
        let b = FrameBudget::unbounded().with_reconcile_nodes(3);
        assert!(
            b.check_reconcile(3).is_none(),
            "exactly at the limit is fine"
        );
    }

    #[test]
    fn over_limit_yields_violation_carrying_counts() {
        let b = FrameBudget::unbounded().with_layout_boxes(2);
        let violation = b.check_layout(5).expect("5 > 2 exceeds the budget");
        assert_eq!(violation.subsystem(), DiagnosticSubsystem::Layout);
        // The count is carried through the nanosecond encoding.
        assert_eq!(violation.budget(), Duration::from_nanos(2));
        assert_eq!(violation.measured(), Duration::from_nanos(5));
        assert_eq!(violation.overage(), Duration::from_nanos(3));
        // It converts to a warning diagnostic on the layout subsystem.
        let diag = violation.to_diagnostic();
        assert_eq!(diag.subsystem, DiagnosticSubsystem::Layout);
    }

    #[test]
    fn each_phase_targets_its_own_subsystem() {
        let b = FrameBudget::unbounded()
            .with_reconcile_nodes(0)
            .with_style_nodes(0)
            .with_layout_boxes(0)
            .with_compose_commands(0);
        assert_eq!(
            b.check_reconcile(1).unwrap().subsystem(),
            DiagnosticSubsystem::Tree
        );
        assert_eq!(
            b.check_style(1).unwrap().subsystem(),
            DiagnosticSubsystem::Style
        );
        assert_eq!(
            b.check_layout(1).unwrap().subsystem(),
            DiagnosticSubsystem::Layout
        );
        assert_eq!(
            b.check_compose(1).unwrap().subsystem(),
            DiagnosticSubsystem::Render
        );
    }
}
