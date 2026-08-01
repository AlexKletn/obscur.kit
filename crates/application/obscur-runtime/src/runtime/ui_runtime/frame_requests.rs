#![doc = include_str!("frame_requests.doc/_module.md")]

use obscur_identity::SignalId;

use crate::frame::FrameReason;
use crate::runtime::{BackgroundServiced, FramePlan};
use crate::scheduler::BackpressureDecision;
use crate::transaction::TransactionCommit;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("frame_requests.doc/request_frame.md")]
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
        self.scheduler.request_reason(reason)
    }

    #[doc = include_str!("frame_requests.doc/apply_transaction.md")]
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

    #[doc = include_str!("frame_requests.doc/notify_signal.md")]
    pub fn notify_signal(&mut self, source: SignalId) -> usize {
        let invalidated = self.components.notify(source);
        if invalidated > 0 {
            self.scheduler.request_reason(FrameReason::UiCommit);
        }
        invalidated
    }

    #[doc = include_str!("frame_requests.doc/peek_frame_reason.md")]
    #[cfg(test)]
    pub(super) fn peek_frame_reason(&self) -> Option<FrameReason> {
        // The path is spelled in full rather than imported: this reader is
        // `#[cfg(test)]`, and it is the only item in the file that names the
        // request type, so an import would be unused in every other profile.
        self.scheduler
            .peek()
            .map(crate::frame::FrameRequest::reason)
    }

    #[doc = include_str!("frame_requests.doc/request_background.md")]
    pub fn request_background(&mut self) {
        if self.background.request() {
            self.scheduler.note_background_pending();
        }
    }

    #[doc = include_str!("frame_requests.doc/service_background_unit.md")]
    pub(super) fn service_background_unit(&mut self) {
        let Some(remaining) = self.background.service() else {
            return;
        };
        // Reset the starvation fairness count: the waiting work just ran.
        self.scheduler.service_background();
        // With no work left, stop treating later frames as preemptions.
        if remaining == BackgroundServiced::Drained {
            self.scheduler.note_background_idle();
        }
    }
}
