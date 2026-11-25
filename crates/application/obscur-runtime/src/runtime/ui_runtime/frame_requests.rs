#![doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_requests.doc/_module.md")]

use obscur_identity::SignalId;
use obscur_task::CancellationToken;
use std::time::Duration;

use crate::frame::FrameReason;
use crate::runtime::{BackgroundBudget, BackgroundServiced, FramePlan};
use crate::scheduler::BackpressureDecision;
use crate::transaction::TransactionCommit;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_requests.doc/request_frame.md")]
    pub fn request_frame(&mut self, reason: FrameReason) -> BackpressureDecision {
        // A request made while the dispatch phase is running is **coalesced into
        // the frame that is running it**: that frame has not chosen its plan yet,
        // so it can serve the request itself. This is not a special case bolted
        // on — it is what §3.5's "updates are coalesced" means, and `Coalesce` is
        // the scheduler vocabulary's own word for it.
        //
        // Routing it to the scheduler instead would leave a request pending that
        // the current frame is about to satisfy, and every burst of input would
        // cost one extra full restyle/relayout/compose that changes nothing.
        if self.input.coalesce(reason) {
            return BackpressureDecision::Coalesce;
        }
        let decision = self.scheduler.request_reason(reason);
        if reason == FrameReason::Animation && decision != BackpressureDecision::Reject {
            self.view_animation_requested = true;
        }
        decision
    }

    /// Request a frame for runtime-owned motion using the retained view.
    pub(super) fn request_timeline_frame(&mut self) {
        self.timeline_waiting_for_tick = false;
        if !self.input.coalesce(FrameReason::Animation) {
            self.scheduler.request_reason(FrameReason::Animation);
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_requests.doc/apply_transaction.md")]
    pub fn apply_transaction(
        &mut self,
        commit: &TransactionCommit,
    ) -> Option<BackpressureDecision> {
        if commit.is_empty() {
            return None;
        }
        let plan = FramePlan::for_transaction(commit);
        // Merge rather than overwrite: two commits between frames both get the
        // phases they asked for. Dropping the earlier one would silently lose the
        // work it classified.
        self.staged_plan = Some(match self.staged_plan {
            Some(pending) => pending.merged_with(plan),
            None => plan,
        });
        Some(self.scheduler.request_reason(FrameReason::UiCommit))
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_requests.doc/notify_signal.md")]
    pub fn notify_signal(&mut self, source: SignalId) -> usize {
        let invalidated = self.components.notify(source);
        if invalidated > 0 {
            self.scheduler.request_reason(FrameReason::UiCommit);
        }
        invalidated
    }

    /// The [`FrameReason`] of the most-urgent pending frame request, if any (test
    /// introspection — lets a test assert which frame a keystroke scheduled).
    #[cfg(test)]
    pub(super) fn peek_frame_reason(&self) -> Option<FrameReason> {
        // The path is spelled in full rather than imported: this reader is
        // `#[cfg(test)]`, and it is the only item in the file that names the
        // request type, so an import would be unused in every other profile.
        self.scheduler
            .peek()
            .map(crate::frame::FrameRequest::reason)
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_requests.doc/request_background.md")]
    pub fn request_background(&mut self) {
        if self.background.request() {
            self.scheduler.note_background_pending();
        }
    }

    /// Sets the runtime's background service limits starting with the next poll.
    pub fn set_background_budget(&mut self, budget: BackgroundBudget) {
        self.background_budget = budget;
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_requests.doc/enqueue_background.md")]
    pub fn enqueue_background(&mut self, job: impl FnOnce() + 'static) {
        if self.background.enqueue(None, None, job) {
            self.scheduler.note_background_pending();
        }
    }

    /// Enqueues a short operation unless the supplied token has already been cancelled.
    /// The runtime checks the token again before invoking the operation. A cancelled
    /// entry is discarded at its next service slot. Cancellation applies to queued
    /// operations. A callback that has already started runs to completion.
    pub fn enqueue_cancellable_background(
        &mut self,
        cancellation: CancellationToken,
        job: impl FnOnce() + 'static,
    ) {
        if !cancellation.is_cancelled() && self.background.enqueue(Some(cancellation), None, job) {
            self.scheduler.note_background_pending();
        }
    }

    /// Enqueues a short operation with a lifetime measured in logical runtime time.
    /// The operation is discarded if its lifetime has elapsed when its slot is
    /// serviced. A zero lifetime skips enqueueing. The host advances this clock
    /// through `UiRuntime::advance_motion`. Once started, the operation runs to completion.
    pub fn enqueue_background_with_deadline(
        &mut self,
        valid_for: Duration,
        job: impl FnOnce() + 'static,
    ) {
        let deadline = self.logical_time.saturating_add(valid_for);
        if deadline > self.logical_time && self.background.enqueue(None, Some(deadline), job) {
            self.scheduler.note_background_pending();
        }
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/frame_requests.doc/service_background_unit.md")]
    pub(super) fn service_background_unit(&mut self) {
        let job = self.background.take_job();
        if job.as_ref().is_some_and(|(token, deadline, _)| {
            token.as_ref().is_some_and(CancellationToken::is_cancelled)
                || deadline.is_some_and(|deadline| self.logical_time >= deadline)
        }) {
            self.background.discard_job();
            if self.background.pending() == 0 {
                self.scheduler.service_background();
                self.scheduler.note_background_idle();
            }
            return;
        }
        let Some(remaining) = self.background.service() else {
            return;
        };
        // Reset the starvation fairness count: the waiting work just ran.
        self.scheduler.service_background();
        // With no work left, stop treating later frames as preemptions.
        if remaining == BackgroundServiced::Drained {
            self.scheduler.note_background_idle();
        }
        if let Some((_, _, job)) = job {
            job();
        }
    }
}
