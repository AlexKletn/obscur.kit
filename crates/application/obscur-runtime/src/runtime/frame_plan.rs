#![doc = include_str!("frame_plan.doc/_module.md")]

use obscur_layout::invalidation::LayoutInvalidation;
use obscur_style::invalidation::StyleInvalidation;

use crate::frame::FrameReason;
use crate::invalidation::{Invalidation, InvalidationReason, InvalidationRouter};
use crate::transaction::{TransactionCommit, UiOperation};

#[doc = include_str!("frame_plan.doc/FramePlan.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FramePlan {
    #[doc = include_str!("frame_plan.doc/reconcile.md")]
    reconcile: bool,
    #[doc = include_str!("frame_plan.doc/style.md")]
    style: bool,
    #[doc = include_str!("frame_plan.doc/layout.md")]
    layout: bool,
    #[doc = include_str!("frame_plan.doc/compose.md")]
    compose: bool,
}

impl FramePlan {
    #[doc = include_str!("frame_plan.doc/for_frame.md")]
    #[must_use]
    pub(crate) fn for_frame(reason: FrameReason, first_frame: bool) -> Self {
        if first_frame {
            return Self::full();
        }

        // A pure text edit into the focused editable that cannot change its box
        // size is compose-only: re-shape + repaint the field, reusing the retained
        // styles and boxes (spec §3.4 phase 5). The router is bypassed because no
        // domain invalidation is involved — only the scene is rebuilt from the
        // live editing session.
        if matches!(reason, FrameReason::TextEdit) {
            return Self::compose_only();
        }

        // Drive the router with the reason's global invalidation cause so its
        // dirty-phase flags decide what this frame recalculates (spec §4.3).
        let mut router = InvalidationRouter::new();
        let mut style = StyleInvalidation::new();
        let mut layout = LayoutInvalidation::new();
        if let Some(cause) = Self::invalidation_for(reason) {
            router.route(Invalidation::global(cause), &mut style, &mut layout);
        }

        Self {
            reconcile: Self::reason_is_structural(reason),
            style: router.needs_style(),
            layout: router.needs_layout(),
            compose: router.needs_scene(),
        }
    }

    #[doc = include_str!("frame_plan.doc/for_transaction.md")]
    #[must_use]
    pub(crate) fn for_transaction(commit: &TransactionCommit) -> Self {
        let mut router = InvalidationRouter::new();
        let mut style = StyleInvalidation::new();
        let mut layout = LayoutInvalidation::new();
        let mut reconcile = false;
        for &op in commit.operations() {
            reconcile |= matches!(op, UiOperation::ReplaceRoot | UiOperation::Rerender { .. });
            router.route(op.invalidation(), &mut style, &mut layout);
        }
        Self {
            reconcile,
            style: router.needs_style(),
            layout: router.needs_layout(),
            compose: router.needs_scene(),
        }
    }

    #[doc = include_str!("frame_plan.doc/merged_with.md")]
    #[must_use]
    pub(crate) const fn merged_with(self, other: Self) -> Self {
        Self {
            reconcile: self.reconcile || other.reconcile,
            style: self.style || other.style,
            layout: self.layout || other.layout,
            compose: self.compose || other.compose,
        }
    }

    #[doc = include_str!("frame_plan.doc/full.md")]
    #[must_use]
    pub(crate) const fn full() -> Self {
        Self {
            reconcile: true,
            style: true,
            layout: true,
            compose: true,
        }
    }

    #[doc = include_str!("frame_plan.doc/compose_only.md")]
    #[must_use]
    pub(crate) const fn compose_only() -> Self {
        Self {
            reconcile: false,
            style: false,
            layout: false,
            compose: true,
        }
    }

    #[doc = include_str!("frame_plan.doc/invalidation_for.md")]
    const fn invalidation_for(reason: FrameReason) -> Option<InvalidationReason> {
        match reason {
            FrameReason::UiCommit
            | FrameReason::ExternalRender
            | FrameReason::ResourceCompletion => Some(InvalidationReason::TreeChanged),
            FrameReason::WindowChange => Some(InvalidationReason::EnvironmentChanged),
            FrameReason::Input => Some(InvalidationReason::PseudoStateChanged),
            FrameReason::Animation => Some(InvalidationReason::StyleChanged),
            // `TextEdit` short-circuits to `compose_only` in `for_frame` before
            // this is ever consulted; map it to `None` to keep the match total.
            FrameReason::TextEdit | FrameReason::DiagnosticsCapture => None,
        }
    }

    #[doc = include_str!("frame_plan.doc/reason_is_structural.md")]
    const fn reason_is_structural(reason: FrameReason) -> bool {
        matches!(
            reason,
            FrameReason::UiCommit | FrameReason::ExternalRender | FrameReason::ResourceCompletion
        )
    }

    #[doc = include_str!("frame_plan.doc/reconcile_2.md")]
    #[must_use]
    pub(crate) const fn reconcile(self) -> bool {
        self.reconcile
    }

    #[doc = include_str!("frame_plan.doc/style_2.md")]
    #[must_use]
    pub(crate) const fn style(self) -> bool {
        self.style
    }

    #[doc = include_str!("frame_plan.doc/layout_2.md")]
    #[must_use]
    pub(crate) const fn layout(self) -> bool {
        self.layout
    }

    #[doc = include_str!("frame_plan.doc/compose_2.md")]
    #[must_use]
    pub(crate) const fn compose(self) -> bool {
        self.compose
    }

    #[doc = include_str!("frame_plan.doc/is_idle.md")]
    #[must_use]
    pub(crate) const fn is_idle(self) -> bool {
        !(self.reconcile || self.style || self.layout || self.compose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::{NodeId, VersionStamp};

    /// A commit of `ops`, as a transaction would produce.
    fn commit(ops: Vec<UiOperation>) -> TransactionCommit {
        TransactionCommit::new(ops, VersionStamp::from_raw(1))
    }

    fn node() -> NodeId {
        NodeId::from_raw(1)
    }

    #[test]
    fn a_style_only_transaction_restyles_without_reconciling() {
        // THE point of classifying from operations instead of the frame reason: a
        // `UiCommit` frame is conservatively full, but the operations know the
        // tree did not change, because none of them said so.
        let plan = FramePlan::for_transaction(&commit(vec![UiOperation::InvalidateStyle {
            node: node(),
        }]));
        assert!(!plan.reconcile(), "no operation claimed a tree change");
        assert!(plan.style());
        assert!(plan.layout(), "a style can feed geometry");
        assert!(plan.compose());
        assert_ne!(
            plan,
            FramePlan::for_frame(FrameReason::UiCommit, false),
            "the whole gain is being cheaper than the reason-derived plan"
        );
    }

    #[test]
    fn a_layout_only_transaction_does_not_dirty_the_style_phase() {
        // The difference between the two invalidate operations, and the reason
        // they are separate: `InvalidateLayout` claims geometry only.
        let plan = FramePlan::for_transaction(&commit(vec![UiOperation::InvalidateLayout {
            node: node(),
        }]));
        assert!(!plan.reconcile());
        assert!(!plan.style(), "geometry went stale, the style did not");
        assert!(plan.layout());
        assert!(plan.compose());
    }

    #[test]
    fn a_root_replacement_reconciles_everything() {
        let plan = FramePlan::for_transaction(&commit(vec![UiOperation::ReplaceRoot]));
        assert_eq!(plan, FramePlan::full());
    }

    #[test]
    fn a_subtree_rerender_reconciles_too() {
        // A component's output can change its subtree's structure.
        let plan =
            FramePlan::for_transaction(&commit(vec![UiOperation::Rerender { node: node() }]));
        assert!(plan.reconcile());
        assert!(plan.style());
        assert!(plan.layout());
        assert!(plan.compose());
    }

    #[test]
    fn an_empty_transaction_is_idle() {
        // What makes a cancelled provisional transaction free: nothing was asked
        // for, so no phase is classified.
        assert!(FramePlan::for_transaction(&commit(Vec::new())).is_idle());
    }

    #[test]
    fn a_mixed_transaction_takes_the_union_of_its_operations() {
        // One structural operation is enough to reconcile the frame, however many
        // cheap ones it travels with.
        let plan = FramePlan::for_transaction(&commit(vec![
            UiOperation::InvalidateLayout { node: node() },
            UiOperation::Rerender { node: node() },
        ]));
        assert!(plan.reconcile(), "the `Rerender` decides");
        assert!(plan.style());
    }

    #[test]
    fn merging_takes_every_phase_either_plan_asked_for() {
        let style_only = FramePlan::for_transaction(&commit(vec![UiOperation::InvalidateStyle {
            node: node(),
        }]));
        assert!(!style_only.reconcile());
        assert_eq!(
            style_only.merged_with(FramePlan::for_transaction(&commit(vec![
                UiOperation::ReplaceRoot
            ]))),
            FramePlan::full(),
            "two commits between frames: the frame owes both"
        );
        // Merging with an idle plan changes nothing.
        assert_eq!(
            style_only.merged_with(FramePlan::for_transaction(&commit(Vec::new()))),
            style_only
        );
    }

    #[test]
    fn first_frame_is_always_full_even_for_input() {
        let plan = FramePlan::for_frame(FrameReason::Input, true);
        assert_eq!(plan, FramePlan::full());
        assert!(plan.reconcile());
        assert!(plan.style());
        assert!(plan.layout());
        assert!(plan.compose());
        assert!(!plan.is_idle());
    }

    #[test]
    fn ui_commit_runs_every_phase() {
        let plan = FramePlan::for_frame(FrameReason::UiCommit, false);
        assert!(plan.reconcile(), "a state commit reconciles the tree");
        assert!(plan.style());
        assert!(plan.layout());
        assert!(plan.compose());
    }

    #[test]
    fn input_restyles_without_reconciling() {
        // The headline win: a hover/focus (Input) frame must NOT re-reconcile,
        // yet must still restyle and re-compose so the pseudo-state reaches paint.
        let plan = FramePlan::for_frame(FrameReason::Input, false);
        assert!(!plan.reconcile(), "an input frame does not reconcile");
        assert!(
            plan.style(),
            "an input frame restyles (pseudo-state changed)"
        );
        assert!(plan.layout());
        assert!(plan.compose());
        assert!(!plan.is_idle());
    }

    #[test]
    fn text_edit_is_compose_only() {
        // A same-size text edit re-composes the field's glyphs + caret without
        // touching the retained tree, styles or boxes: compose runs; reconcile,
        // style and layout are all skipped (the per-keystroke fast path).
        let plan = FramePlan::for_frame(FrameReason::TextEdit, false);
        assert!(!plan.reconcile(), "a text edit does not reconcile");
        assert!(!plan.style(), "a text edit does not restyle");
        assert!(!plan.layout(), "a same-size text edit does not relayout");
        assert!(plan.compose(), "a text edit re-composes the field");
        assert!(!plan.is_idle());
        assert_eq!(plan, FramePlan::compose_only());
    }

    #[test]
    fn text_edit_over_an_empty_tree_is_still_full() {
        // The first-frame guard wins over the compose-only short-circuit: with no
        // retained tree there is nothing to reuse, so every phase must run.
        assert_eq!(
            FramePlan::for_frame(FrameReason::TextEdit, true),
            FramePlan::full()
        );
    }

    #[test]
    fn animation_recomposes_without_reconciling() {
        let plan = FramePlan::for_frame(FrameReason::Animation, false);
        assert!(!plan.reconcile(), "a motion tick does not reconcile");
        assert!(plan.layout(), "an animated value can feed layout");
        assert!(plan.compose());
    }

    #[test]
    fn window_change_restyles_without_reconciling() {
        let plan = FramePlan::for_frame(FrameReason::WindowChange, false);
        assert!(!plan.reconcile(), "a resize does not change tree structure");
        assert!(plan.style(), "media queries re-evaluate on a resize");
        assert!(plan.layout());
        assert!(plan.compose());
    }

    #[test]
    fn diagnostics_capture_is_idle() {
        let plan = FramePlan::for_frame(FrameReason::DiagnosticsCapture, false);
        assert!(plan.is_idle(), "a capture recalculates nothing");
        assert!(!plan.reconcile());
        assert!(!plan.style());
        assert!(!plan.layout());
        assert!(!plan.compose());
    }
}
