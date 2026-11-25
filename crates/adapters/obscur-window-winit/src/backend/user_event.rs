#[doc = include_str!("../../docs/api/backend/winit_event_loop.doc/UserEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserEvent {
    /// Wake the event loop so it can service a scheduled frame.
    WakeUp,
}
