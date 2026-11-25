#![doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_classification.doc/_module.md")]

use crate::frame::{FrameContext, FrameReason};
use crate::runtime::FramePlan;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_classification.doc/plan_for_frame.md")]
    pub(super) fn plan_for_frame(
        &mut self,
        ctx: &FrameContext,
        first_frame: bool,
        dispatched: bool,
        landed: bool,
    ) -> FramePlan {
        // A plan staged by `apply_transaction` wins over the reason-derived one:
        // it was classified from the committed operations themselves, which is
        // what actually changed, rather than from the frame's category — see
        // `FramePlan::for_transaction`. The first-frame guard still overrides
        // both, because there is nothing retained to reuse.
        let mut plan = match self.staged_plan.take() {
            Some(staged) if !first_frame => staged,
            _ => FramePlan::for_frame(ctx.reason(), first_frame),
        };
        // Whatever the dispatch phase invalidated is **merged** in rather than
        // replacing anything: a hover that restyles and a commit that reconciles
        // are both owed by this one frame. Merging is also why dispatch never
        // needs a frame of its own.
        if let Some(from_dispatch) = self.input.take_coalesced() {
            debug_assert!(dispatched, "only the dispatch phase stages a dispatch plan");
            plan = plan.merged_with(from_dispatch);
        }
        // A result committed above changed what this frame would paint, so it is
        // merged in for the same reason a dispatch plan is: the reason the caller
        // asked for the frame does not know a load landed. Without this an
        // `is_idle` frame (a diagnostics capture) would swallow the completion —
        // the result is consumed off the channel, and the retained scene it
        // re-presents was composed before the image existed.
        if landed {
            plan = plan.merged_with(FramePlan::for_frame(
                FrameReason::ResourceCompletion,
                first_frame,
            ));
        }
        plan
    }
}
