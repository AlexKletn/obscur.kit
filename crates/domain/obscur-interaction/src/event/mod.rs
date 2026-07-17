//! Event module.
mod dispatch_result;
pub use dispatch_result::DispatchResult;
mod event_dispatcher;
pub use event_dispatcher::{DispatchStep, EventDispatcher};
mod event_origin;
pub use event_origin::EventOrigin;
mod event_phase;
pub use event_phase::EventPhase;
mod event_propagation;
pub use event_propagation::EventPropagation;
mod event_trust;
pub use event_trust::EventTrust;
mod ui_event;
pub use ui_event::UiEvent;
