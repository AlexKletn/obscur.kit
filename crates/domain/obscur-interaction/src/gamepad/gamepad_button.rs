#![doc = include_str!("gamepad_button.doc/_module.md")]

#[doc = include_str!("gamepad_button.doc/GamepadButton.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    #[doc = include_str!("gamepad_button.doc/South.md")]
    South,
    #[doc = include_str!("gamepad_button.doc/East.md")]
    East,
    #[doc = include_str!("gamepad_button.doc/West.md")]
    West,
    #[doc = include_str!("gamepad_button.doc/North.md")]
    North,
    #[doc = include_str!("gamepad_button.doc/DpadUp.md")]
    DpadUp,
    #[doc = include_str!("gamepad_button.doc/DpadDown.md")]
    DpadDown,
    #[doc = include_str!("gamepad_button.doc/DpadLeft.md")]
    DpadLeft,
    #[doc = include_str!("gamepad_button.doc/DpadRight.md")]
    DpadRight,
    #[doc = include_str!("gamepad_button.doc/LeftShoulder.md")]
    LeftShoulder,
    #[doc = include_str!("gamepad_button.doc/RightShoulder.md")]
    RightShoulder,
    #[doc = include_str!("gamepad_button.doc/LeftTrigger.md")]
    LeftTrigger,
    #[doc = include_str!("gamepad_button.doc/RightTrigger.md")]
    RightTrigger,
    #[doc = include_str!("gamepad_button.doc/LeftStick.md")]
    LeftStick,
    #[doc = include_str!("gamepad_button.doc/RightStick.md")]
    RightStick,
    #[doc = include_str!("gamepad_button.doc/Start.md")]
    Start,
    #[doc = include_str!("gamepad_button.doc/Select.md")]
    Select,
    #[doc = include_str!("gamepad_button.doc/Guide.md")]
    Guide,
}

impl GamepadButton {
    #[doc = include_str!("gamepad_button.doc/is_dpad.md")]
    #[must_use]
    pub const fn is_dpad(&self) -> bool {
        matches!(
            self,
            Self::DpadUp | Self::DpadDown | Self::DpadLeft | Self::DpadRight
        )
    }

    #[doc = include_str!("gamepad_button.doc/is_face.md")]
    #[must_use]
    pub const fn is_face(&self) -> bool {
        matches!(self, Self::South | Self::East | Self::West | Self::North)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_dpad_only_for_dpad_buttons() {
        assert!(GamepadButton::DpadUp.is_dpad());
        assert!(GamepadButton::DpadDown.is_dpad());
        assert!(GamepadButton::DpadLeft.is_dpad());
        assert!(GamepadButton::DpadRight.is_dpad());
        assert!(!GamepadButton::South.is_dpad());
        assert!(!GamepadButton::Start.is_dpad());
    }

    #[test]
    fn is_face_only_for_face_buttons() {
        assert!(GamepadButton::South.is_face());
        assert!(GamepadButton::East.is_face());
        assert!(GamepadButton::West.is_face());
        assert!(GamepadButton::North.is_face());
        assert!(!GamepadButton::DpadUp.is_face());
        assert!(!GamepadButton::LeftShoulder.is_face());
    }
}
