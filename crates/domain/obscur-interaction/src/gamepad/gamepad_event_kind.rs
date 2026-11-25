use super::{GamepadAxis, GamepadButton};

/// The kind of change carried by a [`GamepadEvent`](super::GamepadEvent) (spec §6.16).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamepadEventKind {
    /// A button was pressed.
    ButtonDown(GamepadButton),
    /// A button was released.
    ButtonUp(GamepadButton),
    /// An analog axis moved to `value` (sticks in `-1.0..=1.0`, triggers in
    /// `0.0..=1.0`).
    AxisMotion {
        /// The axis that moved.
        axis: GamepadAxis,
        /// The axis's new value.
        value: f32,
    },
    /// The gamepad was connected.
    Connected,
    /// The gamepad was disconnected.
    Disconnected,
}
