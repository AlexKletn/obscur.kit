//! Cross-thread event-loop waking for frame scheduling (spec §3.5). The
//! [`EventLoopWaker`] wraps a winit event-loop proxy so the frame scheduler can
//! nudge the loop awake when a frame is due, from any thread.

mod event_loop_waker;
pub use event_loop_waker::EventLoopWaker;
