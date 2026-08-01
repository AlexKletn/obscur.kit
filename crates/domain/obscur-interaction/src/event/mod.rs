#![doc = include_str!("mod.doc/_module.md")]
mod dispatch_result;
pub use dispatch_result::DispatchResult;
mod event_dispatcher;
pub use event_dispatcher::{DispatchStep, EventDispatcher};
mod event_origin;
pub use event_origin::EventOrigin;
mod event_payload;
pub use event_payload::EventPayload;
mod event_phase;
pub use event_phase::EventPhase;
mod event_propagation;
pub use event_propagation::EventPropagation;
mod event_trust;
pub use event_trust::EventTrust;
mod listener_phase;
pub use listener_phase::ListenerPhase;
mod ui_event;
pub use ui_event::UiEvent;
