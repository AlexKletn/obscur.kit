//! The [`EventLoopWaker`] cross-thread frame-scheduling waker (spec §3.5 frame
//! scheduling wake).

use winit::event_loop::EventLoopProxy;

use crate::backend::UserEvent;

#[doc = include_str!("../../docs/api/wake/event_loop_waker.doc/EventLoopWaker.md")]
#[derive(Clone)]
pub struct EventLoopWaker {
    /// The winit proxy used to post wake-ups into the loop.
    proxy: EventLoopProxy<UserEvent>,
}

impl EventLoopWaker {
    /// Wraps an event-loop `proxy` (from
    /// [`EventLoop::create_proxy`](winit::event_loop::EventLoop::create_proxy)).
    #[must_use]
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        Self { proxy }
    }

    #[doc = include_str!("../../docs/api/wake/event_loop_waker.doc/wake.md")]
    pub fn wake(&self) {
        let _ = self.proxy.send_event(UserEvent::WakeUp);
    }
}

impl std::fmt::Debug for EventLoopWaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventLoopWaker").finish_non_exhaustive()
    }
}
