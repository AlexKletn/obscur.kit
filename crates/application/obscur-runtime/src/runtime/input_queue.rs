//! [`InputQueue`]: the events awaiting phase 3, and what that phase coalesces.

use std::collections::VecDeque;

use crate::frame::FrameReason;

use super::{FramePlan, InputDispatch, InputEvent};

#[doc = include_str!("../../docs/api/runtime/input_queue.doc/InputQueue.md")]
#[derive(Debug, Default)]
pub(crate) struct InputQueue {
    /// Events received (phase 1) and waiting for dispatch (phase 3), in arrival
    /// order.
    queued: VecDeque<InputEvent>,
    /// What the last frame's dispatch phase produced, awaiting
    /// [`take_dispatches`](Self::take_dispatches).
    dispatches: Vec<InputDispatch>,
    /// Whether the dispatch phase is running, which is what redirects a handler's
    /// own frame request into [`coalesced`](Self::coalesced).
    in_dispatch: bool,
    /// The plan accumulated from frame requests made *during* dispatch, for the
    /// running frame to merge into its own.
    coalesced: Option<FramePlan>,
}

impl InputQueue {
    /// An empty queue with the dispatch phase not running.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Queues `event` for the next frame's dispatch phase.
    pub(crate) fn push(&mut self, event: InputEvent) {
        self.queued.push_back(event);
    }

    /// How many events are waiting for dispatch.
    pub(crate) fn pending(&self) -> usize {
        self.queued.len()
    }

    #[doc = include_str!("../../docs/api/runtime/input_queue.doc/take_dispatches.md")]
    pub(crate) fn take_dispatches(&mut self) -> Vec<InputDispatch> {
        std::mem::take(&mut self.dispatches)
    }

    #[doc = include_str!("../../docs/api/runtime/input_queue.doc/begin_dispatch.md")]
    pub(crate) fn begin_dispatch(&mut self) -> Option<VecDeque<InputEvent>> {
        debug_assert!(!self.in_dispatch, "the dispatch phase is not re-entrant");
        if self.queued.is_empty() {
            self.dispatches.clear();
            return None;
        }
        self.in_dispatch = true;
        Some(std::mem::take(&mut self.queued))
    }

    /// Closes the dispatch phase, recording what its events produced.
    pub(crate) fn end_dispatch(&mut self, dispatches: Vec<InputDispatch>) {
        self.in_dispatch = false;
        self.dispatches = dispatches;
    }

    #[doc = include_str!("../../docs/api/runtime/input_queue.doc/coalesce.md")]
    pub(crate) fn coalesce(&mut self, reason: FrameReason) -> bool {
        if !self.in_dispatch {
            return false;
        }
        let wanted = FramePlan::for_frame(reason, false);
        self.coalesced = Some(match self.coalesced {
            Some(pending) => pending.merged_with(wanted),
            None => wanted,
        });
        true
    }

    /// Takes the plan the dispatch phase accumulated, for the running frame to
    /// merge. Empties it, so the same invalidation cannot be merged twice.
    pub(crate) const fn take_coalesced(&mut self) -> Option<FramePlan> {
        self.coalesced.take()
    }
}

#[cfg(test)]
mod tests {
    use super::InputQueue;
    use crate::frame::FrameReason;
    use crate::runtime::{InputDispatch, InputEvent};
    use obscur_geometry::Point;
    use obscur_identity::NodeId;

    fn scroll() -> InputEvent {
        InputEvent::Scroll {
            delta: 1.0,
            position: Point::new(0.0, 0.0),
        }
    }

    #[test]
    fn an_empty_queue_opens_no_dispatch_phase() {
        let mut q = InputQueue::new();
        assert_eq!(q.pending(), 0);
        assert!(q.begin_dispatch().is_none());
        // The latch stayed down, so a request still belongs to the scheduler.
        assert!(!q.coalesce(FrameReason::Input));
    }

    #[test]
    fn a_request_made_during_dispatch_is_coalesced_and_one_made_outside_is_not() {
        let mut q = InputQueue::new();
        q.push(scroll());
        assert!(!q.coalesce(FrameReason::Input), "the phase has not begun");

        let events = q.begin_dispatch().expect("one event was queued");
        assert_eq!(events.len(), 1);
        assert!(q.coalesce(FrameReason::Input));
        assert!(q.coalesce(FrameReason::Animation));
        q.end_dispatch(Vec::new());

        // The latch is down again the moment the phase closes.
        assert!(!q.coalesce(FrameReason::Input));
        // Both requests are in one plan, and taking it empties it — a second merge
        // would apply the same invalidation twice.
        assert!(q.take_coalesced().is_some());
        assert!(q.take_coalesced().is_none());
    }

    #[test]
    fn beginning_a_dispatch_drains_the_queue() {
        let mut q = InputQueue::new();
        q.push(scroll());
        q.push(scroll());
        assert_eq!(q.pending(), 2);
        assert_eq!(q.begin_dispatch().expect("queued").len(), 2);
        // Drained: a second frame with nothing new queued dispatches nothing.
        assert_eq!(q.pending(), 0);
        q.end_dispatch(Vec::new());
        assert!(q.begin_dispatch().is_none());
    }

    #[test]
    fn each_frames_results_replace_the_previous_frames() {
        let mut q = InputQueue::new();
        q.push(scroll());
        let _ = q.begin_dispatch();
        q.end_dispatch(vec![InputDispatch::Scroll(Some(NodeId::from_raw(1)))]);
        assert_eq!(q.take_dispatches().len(), 1);
        // Taken once: the next read is empty rather than the same results again.
        assert!(q.take_dispatches().is_empty());

        // And a frame that dispatches nothing clears what the last one left.
        q.push(scroll());
        let _ = q.begin_dispatch();
        q.end_dispatch(vec![InputDispatch::Scroll(Some(NodeId::from_raw(1)))]);
        assert!(q.begin_dispatch().is_none());
        assert!(
            q.take_dispatches().is_empty(),
            "a frame with no input clears the previous frame's results"
        );
    }
}
