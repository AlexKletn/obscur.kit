#![doc = include_str!("../../../docs/api/runtime/ui_runtime/pending_work.doc/_module.md")]

use crate::frame::FrameReason;
use crate::frame::FrameRequest;
use crate::runtime::InputDispatch;

use super::UiRuntime;

impl UiRuntime {
    /// Returns whether any frame request is pending in the scheduler.
    #[must_use]
    pub fn has_pending_frame(&self) -> bool {
        self.scheduler.has_pending()
    }

    /// Whether the only pending work is a runtime-owned animation tick.
    #[must_use]
    pub fn has_pending_timeline_frame(&self) -> bool {
        !self.view_animation_requested
            && self.scheduler.pending_count() == 1
            && self.scheduler.peek().map(FrameRequest::reason) == Some(FrameReason::Animation)
    }

    /// Returns the number of pending (coalesced) frame requests.
    #[must_use]
    pub fn pending_frames(&self) -> usize {
        self.scheduler.pending_count()
    }

    /// How many queued input events are waiting for the next frame's dispatch
    /// phase.
    #[must_use]
    pub fn pending_input(&self) -> usize {
        self.input.pending()
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pending_work.doc/take_input_dispatches.md")]
    #[must_use]
    pub fn take_input_dispatches(&mut self) -> Vec<InputDispatch> {
        self.input.take_dispatches()
    }

    /// Returns the number of background task-work units still waiting to be
    /// serviced.
    #[must_use]
    pub const fn pending_background(&self) -> usize {
        self.background.pending()
    }

    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/pending_work.doc/serviced_background.md")]
    #[must_use]
    pub const fn serviced_background(&self) -> u64 {
        self.background.serviced()
    }

    /// Returns whether pending background work has been preempted past the
    /// scheduler's fairness threshold and must be serviced next (§3.6).
    #[must_use]
    pub const fn is_background_starved(&self) -> bool {
        self.scheduler.background_starved()
    }
}
