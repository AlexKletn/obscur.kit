#![doc = include_str!("gamepad_event.doc/_module.md")]

use super::GamepadButton;

#[doc = include_str!("gamepad_event.doc/GamepadId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GamepadId(u64);

impl GamepadId {
    #[doc = include_str!("gamepad_event.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("gamepad_event.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[doc = include_str!("gamepad_event.doc/GamepadAxis.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    #[doc = include_str!("gamepad_event.doc/LeftStickX.md")]
    LeftStickX,
    #[doc = include_str!("gamepad_event.doc/LeftStickY.md")]
    LeftStickY,
    #[doc = include_str!("gamepad_event.doc/RightStickX.md")]
    RightStickX,
    #[doc = include_str!("gamepad_event.doc/RightStickY.md")]
    RightStickY,
    #[doc = include_str!("gamepad_event.doc/LeftTrigger.md")]
    LeftTrigger,
    #[doc = include_str!("gamepad_event.doc/RightTrigger.md")]
    RightTrigger,
}

#[doc = include_str!("gamepad_event.doc/GamepadEventKind.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamepadEventKind {
    #[doc = include_str!("gamepad_event.doc/ButtonDown.md")]
    ButtonDown(GamepadButton),
    #[doc = include_str!("gamepad_event.doc/ButtonUp.md")]
    ButtonUp(GamepadButton),
    #[doc = include_str!("gamepad_event.doc/AxisMotion.md")]
    AxisMotion {
        #[doc = include_str!("gamepad_event.doc/axis.md")]
        axis: GamepadAxis,
        #[doc = include_str!("gamepad_event.doc/value.md")]
        value: f32,
    },
    #[doc = include_str!("gamepad_event.doc/Connected.md")]
    Connected,
    #[doc = include_str!("gamepad_event.doc/Disconnected.md")]
    Disconnected,
}

#[doc = include_str!("gamepad_event.doc/GamepadEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GamepadEvent {
    #[doc = include_str!("gamepad_event.doc/gamepad.md")]
    pub gamepad: GamepadId,
    #[doc = include_str!("gamepad_event.doc/kind.md")]
    pub kind: GamepadEventKind,
    #[doc = include_str!("gamepad_event.doc/timestamp_micros.md")]
    pub timestamp_micros: u64,
}

impl GamepadEvent {
    #[doc = include_str!("gamepad_event.doc/new.md")]
    #[must_use]
    pub const fn new(gamepad: GamepadId, kind: GamepadEventKind) -> Self {
        Self {
            gamepad,
            kind,
            timestamp_micros: 0,
        }
    }

    #[doc = include_str!("gamepad_event.doc/with_timestamp.md")]
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = timestamp_micros;
        self
    }

    #[doc = include_str!("gamepad_event.doc/gamepad_2.md")]
    #[must_use]
    pub const fn gamepad(&self) -> GamepadId {
        self.gamepad
    }

    #[doc = include_str!("gamepad_event.doc/kind_2.md")]
    #[must_use]
    pub const fn kind(&self) -> GamepadEventKind {
        self.kind
    }

    #[doc = include_str!("gamepad_event.doc/timestamp.md")]
    #[must_use]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp_micros
    }

    #[doc = include_str!("gamepad_event.doc/button.md")]
    #[must_use]
    pub const fn button(&self) -> Option<GamepadButton> {
        match self.kind {
            GamepadEventKind::ButtonDown(button) | GamepadEventKind::ButtonUp(button) => {
                Some(button)
            }
            _ => None,
        }
    }

    #[doc = include_str!("gamepad_event.doc/is_connection_change.md")]
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
        let down = GamepadEvent::new(pad(), GamepadEventKind::ButtonDown(GamepadButton::South));
        let up = GamepadEvent::new(pad(), GamepadEventKind::ButtonUp(GamepadButton::North));
        assert_eq!(down.button(), Some(GamepadButton::South));
        assert_eq!(up.button(), Some(GamepadButton::North));
        assert_eq!(
            down.kind(),
            GamepadEventKind::ButtonDown(GamepadButton::South)
        );
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
        assert!(
            !GamepadEvent::new(pad(), GamepadEventKind::ButtonDown(GamepadButton::South))
                .is_connection_change()
        );
    }
}
