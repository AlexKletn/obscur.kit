#![doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_entry.doc/_module.md")]

use obscur_style::matching::Stylesheet;
use obscur_view::view::View;

use crate::frame::{FrameContext, FrameReason, FrameResult};
use crate::runtime::{DesiredView, FramePlan};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_entry.doc/run_due_frame.md")]
    pub fn run_due_frame(&mut self, desired: &View, sheets: &[Stylesheet]) -> Option<FrameResult> {
        if !self.poll_frame_work() {
            return None;
        }
        let request = self.scheduler.next_frame()?;
        // §3.4 step 2 "choose frame reasons" (plural) + §3.5 "updates are
        // coalesced": one frame serves EVERY reason pending in this scheduling
        // window, not one frame per reason. The most-urgent reason (popped first,
        // §3.6 priority order) classifies the frame; the pipeline itself orders
        // the phases within it (input dispatch, then the commit, then the
        // resource-completion drain). Merge the phase plans of all requests
        // so input priority cannot erase a simultaneous structural update.
        let staged_commit = self.staged_plan.is_some();
        let mut plan = self.staged_plan.take();
        let mut next = Some(request);
        while let Some(pending) = next {
            let reason = pending.reason();
            if reason != FrameReason::UiCommit || !staged_commit {
                let phases = FramePlan::for_frame(reason, false);
                plan = Some(plan.map_or(phases, |previous| previous.merged_with(phases)));
            }
            next = self.scheduler.take_coalesced_request();
        }
        self.staged_plan = plan;
        self.view_animation_requested = false;
        let ctx = FrameContext::new(self.next_frame, request.reason(), self.surface.viewport())
            .with_environment(self.environment);
        Some(self.run_frame(ctx, DesiredView::Ready(desired), sheets))
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_entry.doc/run_due_frame_reactive.md")]
    pub fn run_due_frame_reactive(&mut self) -> Option<FrameResult> {
        // No builder installed: the reactive path is a no-op and leaves any
        // pending request untouched (a `&View`-taking caller can still drain it).
        self.root_view.as_ref()?;
        if !self.poll_frame_work() {
            return None;
        }
        // Nothing due: no side effects, no rebuild.
        let timeline_only = self.has_pending_timeline_frame();
        let request = self.scheduler.next_frame()?;
        let reason = if timeline_only || request.reason() == FrameReason::DiagnosticsCapture {
            request.reason()
        } else {
            FrameReason::UiCommit
        };
        self.view_animation_requested = false;
        // Coalesce the whole scheduling window into this one frame (§3.4 step 2,
        // §3.5), exactly as the `&View` loop does.
        self.scheduler.coalesce_pending();

        // Take the builder and the held stylesheets out so the `FnMut` call and
        // the `&mut self` render below do not overlap the `&mut`/`&` field borrows;
        // both are restored before returning. `take()` yields `Some` here (the
        // builder was just observed present, and nothing above removes it).
        let mut builder = self.root_view.take()?;
        let sheets = self.sheets.take_reactive();

        // Supply the builder lazily so input or a resource that lands while
        // processing a timeline tick can still promote it to a structural frame.
        let ctx = FrameContext::new(self.next_frame, reason, self.surface.viewport())
            .with_environment(self.environment);
        let result = self.run_frame(ctx, DesiredView::Build(&mut *builder), &sheets);

        self.sheets.set_reactive(sheets);
        self.root_view = Some(builder);
        Some(result)
    }

    /// Polls for a due frame and applies the background budget for idle time or
    /// for a due frame that has reached the fairness threshold.
    fn poll_frame_work(&mut self) -> bool {
        self.poll_accessibility_actions();
        let due = self.has_due_frame();
        let units = if !due {
            self.background_budget.idle_units
        } else if self.scheduler.background_starved() {
            self.background_budget.fairness_units
        } else {
            0
        };
        self.service_background_units(units);
        due
    }

    /// Services the configured idle-work budget while no frame is due.
    /// Returns the number of work units removed from the queue.
    pub fn service_idle_work(&mut self) -> usize {
        if self.has_due_frame() {
            return 0;
        }
        self.service_background_units(self.background_budget.idle_units)
    }

    /// Reports whether a queued frame is ready to run at the current runtime time.
    #[must_use]
    pub fn has_due_frame(&self) -> bool {
        self.scheduler.has_pending()
            && !(self.timeline_waiting_for_tick && self.has_pending_timeline_frame())
    }

    fn service_background_units(&mut self, units: usize) -> usize {
        for serviced in 0..units {
            if self.background.pending() == 0 {
                return serviced;
            }
            self.service_background_unit();
        }
        units
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_entry.doc/render_now.md")]
    pub fn render_now(&mut self, desired: &View, sheets: &[Stylesheet]) -> FrameResult {
        self.poll_accessibility_actions();
        let ctx = FrameContext::new(
            self.next_frame,
            FrameReason::UiCommit,
            self.surface.viewport(),
        )
        .with_environment(self.environment);
        self.run_frame(ctx, DesiredView::Ready(desired), sheets)
    }
}
