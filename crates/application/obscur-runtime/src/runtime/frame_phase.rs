use obscur_diagnostics::{BudgetViolation, DiagnosticSubsystem};

use super::FrameBudget;

#[doc = include_str!("../../docs/api/runtime/frame_telemetry.doc/FramePhase.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FramePhase {
    /// Reconcile: the retained-tree node count after the plan is committed.
    Reconcile,
    /// Style: the computed styles this frame actually **resolved**, which on the
    /// incremental path is not the size of the resulting map.
    Style,
    /// Layout: the arranged boxes.
    Layout,
    /// Compose: the paint commands in the produced display list.
    Compose,
}

impl FramePhase {
    /// The span name, the counter label and the subsystem, which are one choice
    /// and are therefore made in one place: a phase's span and its counter must
    /// name the same subsystem or an observer cannot attribute the cost to the
    /// span it belongs to.
    pub(super) const fn names(self) -> (&'static str, &'static str, DiagnosticSubsystem) {
        match self {
            Self::Reconcile => ("reconcile", "reconcile.nodes", DiagnosticSubsystem::Tree),
            Self::Style => ("style", "style.resolved", DiagnosticSubsystem::Style),
            Self::Layout => ("layout", "layout.boxes", DiagnosticSubsystem::Layout),
            Self::Compose => ("compose", "compose.commands", DiagnosticSubsystem::Render),
        }
    }

    #[doc = include_str!("../../docs/api/runtime/frame_telemetry.doc/check.md")]
    pub(super) fn check(self, budget: FrameBudget, metric: u64) -> Option<BudgetViolation> {
        match self {
            Self::Reconcile => budget.check_reconcile(metric),
            Self::Style => budget.check_style(metric),
            Self::Layout => budget.check_layout(metric),
            Self::Compose => budget.check_compose(metric),
        }
    }
}
