#![doc = include_str!("queued_input.doc/_module.md")]

use crate::frame::FrameReason;
use crate::runtime::{InputDispatch, InputEvent};
use crate::scheduler::BackpressureDecision;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("queued_input.doc/queue_input.md")]
    pub fn queue_input(&mut self, event: InputEvent) -> BackpressureDecision {
        self.input.push(event);
        self.request_frame(FrameReason::Input)
    }

    #[doc = include_str!("queued_input.doc/dispatch_queued_input.md")]
    pub(super) fn dispatch_queued_input(&mut self) -> bool {
        // Opening the phase takes the queue and raises the latch that redirects
        // the dispatchers' own `request_frame` calls into this frame's plan;
        // `end_dispatch` below is the only thing that lowers it again.
        let Some(events) = self.input.begin_dispatch() else {
            return false;
        };
        let mut dispatches = Vec::with_capacity(events.len());
        for event in events {
            dispatches.push(match event {
                InputEvent::Pointer(event) => InputDispatch::Pointer(self.handle_pointer(event)),
                InputEvent::Key(event) => InputDispatch::Key(self.handle_key(event)),
                InputEvent::Composition(event) => {
                    InputDispatch::Composition(self.handle_ime(event))
                }
                InputEvent::Scroll { delta, position } => {
                    InputDispatch::Scroll(self.handle_scroll(delta, position))
                }
            });
        }
        self.input.end_dispatch(dispatches);
        true
    }
}
