//! Gamepad module.
mod gamepad_button;
pub use gamepad_button::GamepadButton;
mod gamepad_event;
pub use gamepad_event::{GamepadAxis, GamepadEvent, GamepadEventKind, GamepadId};
mod haptic_request;
pub use haptic_request::{HapticPattern, HapticRequest};
