//! Per-frame pipeline-phase classification: [`FramePlan`].
//!
//! Spec §3.4 phase 5 ("classify invalidation") requires the frame loop to decide
//! *which* pipeline phases a frame actually needs instead of re-running
//! reconcile → match → style → layout → compose unconditionally. `FramePlan` is
//! that decision: it maps the frame's [`FrameReason`] through the live
//! [`InvalidationRouter`] to the set of phases the runtime must run this frame.

use obscur_layout::invalidation::LayoutInvalidation;
use obscur_style::invalidation::StyleInvalidation;

use crate::frame::FrameReason;
use crate::invalidation::{Invalidation, InvalidationReason, InvalidationRouter};

/// The pipeline phases a single frame must run, classified from its
/// [`FrameReason`] via the [`InvalidationRouter`] (spec §3.4 phase 5, §4.3).
///
/// Each frame the runtime builds a `FramePlan` and gates its phases on it: a
/// phase whose inputs are unchanged is skipped and reuses the previous frame's
/// retained output. The classification is conservative — when a phase cannot be
/// proven unnecessary it is run — so correctness is preserved while clearly
/// redundant work (e.g. re-reconciling on a hover-only frame) is elided.
///
/// The mapping from reason to invalidation cause is:
///
/// | [`FrameReason`]        | [`InvalidationReason`]   | reconcile |
/// |------------------------|--------------------------|-----------|
/// | `UiCommit`             | `TreeChanged`            | yes       |
/// | `ExternalRender`       | `TreeChanged`            | yes       |
/// | `ResourceCompletion`   | `TreeChanged`            | yes       |
/// | `WindowChange`         | `EnvironmentChanged`     | no        |
/// | `Input`                | `PseudoStateChanged`     | no        |
/// | `Animation`            | `StyleChanged`          | no        |
/// | `DiagnosticsCapture`   | *(none — idle)*          | no        |
///
/// The style/layout/compose flags are then read straight off the router's
/// dirty-phase flags, so the router — previously orphaned — is the live source
/// of truth for what a frame recalculates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FramePlan {
    /// The desired view must be reconciled into the retained tree.
    reconcile: bool,
    /// Styles must be re-matched and re-resolved.
    style: bool,
    /// Layout must be re-run.
    layout: bool,
    /// The scene must be re-composed.
    compose: bool,
}

impl FramePlan {
    /// Classifies the phases a frame produced for `reason` must run.
    ///
    /// When `first_frame` is set (the retained tree has never been built) the
    /// plan is [`full`](Self::full) regardless of reason: the first frame — and
    /// any frame over an empty tree — must run every phase so the tree, styles,
    /// boxes and scene all exist before later frames can reuse them.
    #[must_use]
    pub(crate) fn for_frame(reason: FrameReason, first_frame: bool) -> Self {
        if first_frame {
            return Self::full();
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

    /// The full plan: every phase runs (first frame / structural rebuild).
    #[must_use]
    pub(crate) const fn full() -> Self {
        Self {
            reconcile: true,
            style: true,
            layout: true,
            compose: true,
        }
    }

    /// Maps a [`FrameReason`] to the global [`InvalidationReason`] the router
    /// classifies for it, or `None` for an idle reason that recalculates nothing.
    ///
    /// The structural reasons (`UiCommit`/`ExternalRender`/`ResourceCompletion`)
    /// map to [`TreeChanged`](InvalidationReason::TreeChanged) — the deepest
    /// non-environment cause — so they dirty every phase; `WindowChange` maps to
    /// [`EnvironmentChanged`](InvalidationReason::EnvironmentChanged) so media
    /// queries re-evaluate without a tree diff; `Input` maps to
    /// [`PseudoStateChanged`](InvalidationReason::PseudoStateChanged) so a
    /// hover/focus change restyles without reconciling; `Animation` maps to
    /// [`StyleChanged`](InvalidationReason::StyleChanged) (conservative: an
    /// animated value can feed layout). `DiagnosticsCapture` maps to `None` — the
    /// frame only re-presents the retained scene.
    const fn invalidation_for(reason: FrameReason) -> Option<InvalidationReason> {
        match reason {
            FrameReason::UiCommit
            | FrameReason::ExternalRender
            | FrameReason::ResourceCompletion => Some(InvalidationReason::TreeChanged),
            FrameReason::WindowChange => Some(InvalidationReason::EnvironmentChanged),
            FrameReason::Input => Some(InvalidationReason::PseudoStateChanged),
            FrameReason::Animation => Some(InvalidationReason::StyleChanged),
            FrameReason::DiagnosticsCapture => None,
        }
    }

    /// Whether `reason` can change the retained tree's structure and therefore
    /// requires a reconcile pass.
    ///
    /// Only a state commit, an external render request, or a resource completion
    /// can alter the desired tree; input, animation, window and diagnostics
    /// frames leave the structure untouched, so reconcile (an idempotent no-op
    /// for them) is skipped.
    const fn reason_is_structural(reason: FrameReason) -> bool {
        matches!(
            reason,
            FrameReason::UiCommit
                | FrameReason::ExternalRender
                | FrameReason::ResourceCompletion
        )
    }

    /// Whether the reconcile phase must run this frame.
    #[must_use]
    pub(crate) const fn reconcile(self) -> bool {
        self.reconcile
    }

    /// Whether the match + style phase must run this frame.
    #[must_use]
    pub(crate) const fn style(self) -> bool {
        self.style
    }

    /// Whether the layout phase must run this frame.
    #[must_use]
    pub(crate) const fn layout(self) -> bool {
        self.layout
    }

    /// Whether the scene must be re-composed this frame.
    #[must_use]
    pub(crate) const fn compose(self) -> bool {
        self.compose
    }

    /// Whether the frame recalculates nothing: no phase runs and the previous
    /// frame's retained scene is simply re-presented.
    #[must_use]
    pub(crate) const fn is_idle(self) -> bool {
        !(self.reconcile || self.style || self.layout || self.compose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(plan.style(), "an input frame restyles (pseudo-state changed)");
        assert!(plan.layout());
        assert!(plan.compose());
        assert!(!plan.is_idle());
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
