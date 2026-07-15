//! Per-phase frame budgets: [`FrameBudget`].
//!
//! Spec §7.9 requires that exceeding a performance budget produce a diagnostic,
//! not a crash. The runtime is deliberately **clock-free** (the scheduler and
//! frame loop observe no wall-clock; determinism is a hard requirement), so a
//! frame phase has no wall-clock duration to budget. Instead each phase reports
//! a deterministic *cost count* — the number of nodes it processed, boxes it
//! arranged, or paint commands it produced — and this budget bounds that count.
//!
//! To reuse the tested [`PerformanceBudget`]/[`BudgetViolation`] machinery
//! (which is expressed in [`Duration`]), a cost count is encoded as a duration
//! at the rate **one item = one nanosecond**. The encoding is purely a carrier:
//! [`BudgetViolation::overage`](obscur_diagnostics::BudgetViolation::overage) and
//! [`ratio`](obscur_diagnostics::BudgetViolation::ratio) then read back as item
//! counts and count ratios, and the emitted
//! [`Diagnostic`](obscur_diagnostics::Diagnostic) flows through the normal
//! diagnostics channel unchanged.

use std::time::Duration;

use obscur_diagnostics::{BudgetViolation, DiagnosticSubsystem, PerformanceBudget};

/// A bound on each frame phase's deterministic cost count (spec §7.9).
///
/// A `None` limit means the phase is unbounded (the default): no violation is
/// ever produced for it. A `Some(limit)` phase yields a [`BudgetViolation`] when
/// the phase's reported count strictly exceeds `limit`.
///
/// Budgets are attributed to the same subsystems the frame phases record spans
/// under: reconcile → [`Tree`](DiagnosticSubsystem::Tree), style →
/// [`Style`](DiagnosticSubsystem::Style), layout →
/// [`Layout`](DiagnosticSubsystem::Layout), compose →
/// [`Render`](DiagnosticSubsystem::Render).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameBudget {
    /// Maximum retained-tree node count after reconcile, if bounded.
    reconcile_nodes: Option<u64>,
    /// Maximum resolved computed-style count, if bounded.
    style_nodes: Option<u64>,
    /// Maximum arranged layout-box count, if bounded.
    layout_boxes: Option<u64>,
    /// Maximum composed paint-command count, if bounded.
    compose_commands: Option<u64>,
}

impl FrameBudget {
    /// A budget that bounds nothing: every phase is unbounded (the default).
    #[must_use]
    pub const fn unbounded() -> Self {
        Self {
            reconcile_nodes: None,
            style_nodes: None,
            layout_boxes: None,
            compose_commands: None,
        }
    }

    /// Bounds the reconcile phase to at most `max` retained-tree nodes.
    #[must_use]
    pub const fn with_reconcile_nodes(mut self, max: u64) -> Self {
        self.reconcile_nodes = Some(max);
        self
    }

    /// Bounds the style phase to at most `max` resolved computed styles.
    #[must_use]
    pub const fn with_style_nodes(mut self, max: u64) -> Self {
        self.style_nodes = Some(max);
        self
    }

    /// Bounds the layout phase to at most `max` arranged boxes.
    #[must_use]
    pub const fn with_layout_boxes(mut self, max: u64) -> Self {
        self.layout_boxes = Some(max);
        self
    }

    /// Bounds the compose phase to at most `max` paint commands.
    #[must_use]
    pub const fn with_compose_commands(mut self, max: u64) -> Self {
        self.compose_commands = Some(max);
        self
    }

    /// Checks the reconcile phase's `nodes` count, yielding a violation when it
    /// exceeds the configured bound.
    #[must_use]
    pub(crate) fn check_reconcile(self, nodes: u64) -> Option<BudgetViolation> {
        Self::check(self.reconcile_nodes, DiagnosticSubsystem::Tree, nodes)
    }

    /// Checks the style phase's resolved-style `count`.
    #[must_use]
    pub(crate) fn check_style(self, count: u64) -> Option<BudgetViolation> {
        Self::check(self.style_nodes, DiagnosticSubsystem::Style, count)
    }

    /// Checks the layout phase's arranged-box `count`.
    #[must_use]
    pub(crate) fn check_layout(self, count: u64) -> Option<BudgetViolation> {
        Self::check(self.layout_boxes, DiagnosticSubsystem::Layout, count)
    }

    /// Checks the compose phase's paint-command `count`.
    #[must_use]
    pub(crate) fn check_compose(self, count: u64) -> Option<BudgetViolation> {
        Self::check(self.compose_commands, DiagnosticSubsystem::Render, count)
    }

    /// Checks `measured` (a cost count) against an optional `limit` for
    /// `subsystem`, encoding both as nanosecond-durations so the tested
    /// [`PerformanceBudget`] machinery applies.
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
