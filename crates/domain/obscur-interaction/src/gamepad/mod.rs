//! Gamepad module.
mod gamepad_button;
pub use gamepad_button::GamepadButton;
mod gamepad_event;
pub use gamepad_event::GamepadEvent;
mod gamepad_id;
pub use gamepad_id::GamepadId;
mod gamepad_axis;
pub use gamepad_axis::GamepadAxis;
mod gamepad_event_kind;
pub use gamepad_event_kind::GamepadEventKind;
mod haptic_request;
pub use haptic_request::HapticRequest;
mod haptic_pattern;
pub use haptic_pattern::HapticPattern;
