#![doc = include_str!("event_loop_waker.doc/_module.md")]

use winit::event_loop::EventLoopProxy;

use crate::backend::UserEvent;

#[doc = include_str!("event_loop_waker.doc/EventLoopWaker.md")]
#[derive(Clone)]
pub struct EventLoopWaker {
    #[doc = include_str!("event_loop_waker.doc/proxy.md")]
    proxy: EventLoopProxy<UserEvent>,
}

impl EventLoopWaker {
    #[doc = include_str!("event_loop_waker.doc/new.md")]
    #[must_use]
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        Self { proxy }
    }

    #[doc = include_str!("event_loop_waker.doc/wake.md")]
    pub fn wake(&self) {
        let _ = self.proxy.send_event(UserEvent::WakeUp);
    }
}

impl std::fmt::Debug for EventLoopWaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventLoopWaker").finish_non_exhaustive()
    }
}
