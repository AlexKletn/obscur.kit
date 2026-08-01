#![doc = include_str!("frame_entry.doc/_module.md")]

use obscur_style::matching::Stylesheet;
use obscur_view::view::View;

use crate::frame::{FrameContext, FrameReason, FrameResult};
use crate::runtime::DesiredView;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("frame_entry.doc/run_due_frame.md")]
    pub fn run_due_frame(&mut self, desired: &View, sheets: &[Stylesheet]) -> Option<FrameResult> {
        // Starvation prevention (§3.6): if a stream of higher-priority frames has
        // preempted pending background work past the scheduler's fairness
        // threshold, promote and service one background unit *before* pulling the
        // next frame. This drives the scheduler's background band on the live
        // path — `background_starved`/`service_background` — so background work
        // cannot be indefinitely starved by higher-priority frames. The frame
        // priority order itself is untouched: a due frame still runs this call.
        if self.scheduler.background_starved() {
            self.service_background_unit();
        }
        let request = self.scheduler.next_frame()?;
        // §3.4 step 2 "choose frame reasons" (plural) + §3.5 "updates are
        // coalesced": one frame serves EVERY reason pending in this scheduling
        // window, not one frame per reason. The most-urgent reason (popped first,
        // §3.6 priority order) classifies the frame; the pipeline itself orders
        // the phases within it (input dispatch, then the commit, then the
        // resource-completion drain), and dispatch/staged-plan/landed-result each
        // fold their own reason's work into this same frame. Drain the rest so a
        // three-reason window is one frame, not three.
        self.scheduler.coalesce_pending();
        let ctx = FrameContext::new(self.next_frame, request.reason(), self.surface.viewport())
            .with_environment(self.environment);
        Some(self.run_frame(ctx, DesiredView::Ready(desired), sheets))
    }

    #[doc = include_str!("frame_entry.doc/run_due_frame_reactive.md")]
    pub fn run_due_frame_reactive(&mut self) -> Option<FrameResult> {
        // No builder installed: the reactive path is a no-op and leaves any
        // pending request untouched (a `&View`-taking caller can still drain it).
        self.root_view.as_ref()?;
        // Starvation prevention (§3.6), identical to `run_due_frame`.
        if self.scheduler.background_starved() {
            self.service_background_unit();
        }
        // Nothing due: no side effects, no rebuild.
        self.scheduler.next_frame()?;
        // Coalesce the whole scheduling window into this one frame (§3.4 step 2,
        // §3.5), exactly as the `&View` loop does.
        self.scheduler.coalesce_pending();

        // Take the builder and the held stylesheets out so the `FnMut` call and
        // the `&mut self` render below do not overlap the `&mut`/`&` field borrows;
        // both are restored before returning. `take()` yields `Some` here (the
        // builder was just observed present, and nothing above removes it).
        let mut builder = self.root_view.take()?;
        let sheets = self.sheets.take_reactive();

        // A reactive frame rebuilt the view from current state, so it is a UI
        // transaction commit: classify it structurally so the rebuilt view is
        // reconciled into the tree (see the doc comment). The builder is handed to
        // the frame as a lazy `DesiredView::Build` so it runs AFTER the frame's
        // input-dispatch phase: a click handler that commits state has then
        // already run, and the view is built from the state it left behind.
        let ctx = FrameContext::new(
            self.next_frame,
            FrameReason::UiCommit,
            self.surface.viewport(),
        )
        .with_environment(self.environment);
        let result = self.run_frame(ctx, DesiredView::Build(&mut *builder), &sheets);

        self.sheets.set_reactive(sheets);
        self.root_view = Some(builder);
        Some(result)
    }

    #[doc = include_str!("frame_entry.doc/render_now.md")]
    pub fn render_now(&mut self, desired: &View, sheets: &[Stylesheet]) -> FrameResult {
        let ctx = FrameContext::new(
            self.next_frame,
            FrameReason::UiCommit,
            self.surface.viewport(),
        )
        .with_environment(self.environment);
        self.run_frame(ctx, DesiredView::Ready(desired), sheets)
    }
}
