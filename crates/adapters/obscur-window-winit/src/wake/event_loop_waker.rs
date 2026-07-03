//! The [`EventLoopWaker`] cross-thread frame-scheduling waker (spec §3.5 frame
//! scheduling wake).

use winit::event_loop::EventLoopProxy;

use crate::backend::UserEvent;

/// A cheap, cloneable handle that wakes the winit event loop from any thread to
/// request a frame (spec §3.5).
///
/// The §3.5 frame scheduler runs off the event loop and must be able to nudge it
/// awake when a frame becomes due — including from a timer thread. This wraps a
/// winit [`EventLoopProxy`], the one winit primitive that is `Send` and can post
/// into the loop from outside it. [`EventLoopWaker::wake`] sends a
/// [`UserEvent::WakeUp`], which the
/// [`WinitAppHandler`](crate::backend::WinitAppHandler) turns into redraw
/// requests. Being `Clone`, a waker can be handed to as many schedulers or
/// worker threads as needed.
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

    /// Wakes the event loop to service a scheduled frame (spec §3.5).
    ///
    /// A send failure means the loop has already exited; there is nothing left to
    /// wake, so the error is deliberately ignored.
    pub fn wake(&self) {
        let _ = self.proxy.send_event(UserEvent::WakeUp);
    }
}

impl std::fmt::Debug for EventLoopWaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventLoopWaker").finish_non_exhaustive()
    }
}
