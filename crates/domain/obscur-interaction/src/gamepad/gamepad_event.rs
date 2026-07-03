//! The [`GamepadEvent`] input event, its [`GamepadId`] identity,
//! [`GamepadAxis`] axes, and [`GamepadEventKind`] discriminant
//! (spec §6.16 gamepad input).

use super::GamepadButton;

/// Stable identity for one connected gamepad (spec §6.16).
///
/// This is a *local* newtype: the interaction subsystem mints and compares
/// these handles itself. The numeric value carries no meaning outside the
/// subsystem that mints it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GamepadId(u64);

impl GamepadId {
    /// Wraps a raw handle value into a `GamepadId`.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw handle value backing this identifier.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// A continuous analog axis on a gamepad (spec §6.16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    /// The left analog stick's horizontal axis.
    LeftStickX,
    /// The left analog stick's vertical axis.
    LeftStickY,
    /// The right analog stick's horizontal axis.
    RightStickX,
    /// The right analog stick's vertical axis.
    RightStickY,
    /// The left trigger's analog travel.
    LeftTrigger,
    /// The right trigger's analog travel.
    RightTrigger,
}

/// The kind of change carried by a [`GamepadEvent`] (spec §6.16).
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

/// A single input event from a gamepad (spec §6.16).
///
/// Each event pairs the [`gamepad`](Self::gamepad) that produced it with a
/// [`kind`](Self::kind) of change and a logical [`timestamp`](Self::timestamp)
/// supplied by the caller — this crate never reads a clock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GamepadEvent {
    /// The gamepad that produced this event.
    pub gamepad: GamepadId,
    /// The kind of change this event describes.
    pub kind: GamepadEventKind,
    /// A logical timestamp in microseconds, supplied by the caller.
    pub timestamp_micros: u64,
}

impl GamepadEvent {
    /// Creates an event for `gamepad` with the given `kind` and a zero
    /// timestamp.
    #[must_use]
    pub const fn new(gamepad: GamepadId, kind: GamepadEventKind) -> Self {
        Self {
            gamepad,
            kind,
            timestamp_micros: 0,
        }
    }

    /// Returns this event with its logical timestamp set (fluent).
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = timestamp_micros;
        self
    }

    /// Returns the gamepad that produced this event.
    #[must_use]
    pub const fn gamepad(&self) -> GamepadId {
        self.gamepad
    }

    /// Returns the kind of change this event describes.
    #[must_use]
    pub const fn kind(&self) -> GamepadEventKind {
        self.kind
    }

    /// Returns the logical timestamp, in microseconds, supplied by the caller.
    #[must_use]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp_micros
    }

    /// Returns the button involved, if this is a button event
    /// ([`ButtonDown`](GamepadEventKind::ButtonDown) or
    /// [`ButtonUp`](GamepadEventKind::ButtonUp)).
    #[must_use]
    pub const fn button(&self) -> Option<GamepadButton> {
        match self.kind {
            GamepadEventKind::ButtonDown(button) | GamepadEventKind::ButtonUp(button) => {
                Some(button)
            }
            _ => None,
        }
    }

    /// Returns whether this event is a connection change
    /// ([`Connected`](GamepadEventKind::Connected) or
    /// [`Disconnected`](GamepadEventKind::Disconnected)).
    #[must_use]
    pub const fn is_connection_change(&self) -> bool {
        matches!(
            self.kind,
            GamepadEventKind::Connected | GamepadEventKind::Disconnected
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pad() -> GamepadId {
        GamepadId::from_raw(1)
    }

    #[test]
    fn id_round_trips_raw() {
        assert_eq!(GamepadId::from_raw(42).raw(), 42);
    }

    #[test]
    fn new_defaults_timestamp_zero() {
        let e = GamepadEvent::new(pad(), GamepadEventKind::Connected);
        assert_eq!(e.gamepad(), pad());
        assert_eq!(e.timestamp(), 0);
    }

    #[test]
    fn with_timestamp_sets_it() {
        let e = GamepadEvent::new(pad(), GamepadEventKind::Connected).with_timestamp(500);
        assert_eq!(e.timestamp(), 500);
    }

    #[test]
    fn button_reports_button_for_button_events() {
        let down = GamepadEvent::new(
            pad(),
            GamepadEventKind::ButtonDown(GamepadButton::South),
        );
        let up = GamepadEvent::new(pad(), GamepadEventKind::ButtonUp(GamepadButton::North));
        assert_eq!(down.button(), Some(GamepadButton::South));
        assert_eq!(up.button(), Some(GamepadButton::North));
        assert_eq!(down.kind(), GamepadEventKind::ButtonDown(GamepadButton::South));
    }

    #[test]
    fn button_is_none_for_non_button_events() {
        let motion = GamepadEvent::new(
            pad(),
            GamepadEventKind::AxisMotion {
                axis: GamepadAxis::LeftStickX,
                value: 0.5,
            },
        );
        assert_eq!(motion.button(), None);
        if let GamepadEventKind::AxisMotion { axis, value } = motion.kind() {
            assert_eq!(axis, GamepadAxis::LeftStickX);
            assert_eq!(value, 0.5);
        } else {
            panic!("expected axis motion");
        }
    }

    #[test]
    fn is_connection_change_for_connect_disconnect() {
        assert!(GamepadEvent::new(pad(), GamepadEventKind::Connected).is_connection_change());
        assert!(GamepadEvent::new(pad(), GamepadEventKind::Disconnected).is_connection_change());
        assert!(!GamepadEvent::new(
            pad(),
            GamepadEventKind::ButtonDown(GamepadButton::South)
        )
        .is_connection_change());
    }
}
