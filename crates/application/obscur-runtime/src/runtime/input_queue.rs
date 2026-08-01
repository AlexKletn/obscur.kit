#![doc = include_str!("input_queue.doc/_module.md")]

use std::collections::VecDeque;

use crate::frame::FrameReason;

use super::{FramePlan, InputDispatch, InputEvent};

#[doc = include_str!("input_queue.doc/InputQueue.md")]
#[derive(Debug, Default)]
pub(crate) struct InputQueue {
    #[doc = include_str!("input_queue.doc/queued.md")]
    queued: VecDeque<InputEvent>,
    #[doc = include_str!("input_queue.doc/dispatches.md")]
    dispatches: Vec<InputDispatch>,
    #[doc = include_str!("input_queue.doc/in_dispatch.md")]
    in_dispatch: bool,
    #[doc = include_str!("input_queue.doc/coalesced.md")]
    coalesced: Option<FramePlan>,
}

impl InputQueue {
    #[doc = include_str!("input_queue.doc/new.md")]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("input_queue.doc/push.md")]
    pub(crate) fn push(&mut self, event: InputEvent) {
        self.queued.push_back(event);
    }

    #[doc = include_str!("input_queue.doc/pending.md")]
    pub(crate) fn pending(&self) -> usize {
        self.queued.len()
    }

    #[doc = include_str!("input_queue.doc/take_dispatches.md")]
    pub(crate) fn take_dispatches(&mut self) -> Vec<InputDispatch> {
        std::mem::take(&mut self.dispatches)
    }

    #[doc = include_str!("input_queue.doc/begin_dispatch.md")]
    pub(crate) fn begin_dispatch(&mut self) -> Option<VecDeque<InputEvent>> {
        debug_assert!(!self.in_dispatch, "the dispatch phase is not re-entrant");
        if self.queued.is_empty() {
            self.dispatches.clear();
            return None;
        }
        self.in_dispatch = true;
        Some(std::mem::take(&mut self.queued))
    }

    #[doc = include_str!("input_queue.doc/end_dispatch.md")]
    pub(crate) fn end_dispatch(&mut self, dispatches: Vec<InputDispatch>) {
        self.in_dispatch = false;
        self.dispatches = dispatches;
    }

    #[doc = include_str!("input_queue.doc/coalesce.md")]
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

    #[doc = include_str!("input_queue.doc/take_coalesced.md")]
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
