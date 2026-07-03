//! The [`GamepadButton`] enumeration of layout-neutral gamepad buttons
//! (spec §6.16 gamepad input).

/// A physical button on a gamepad, named by position rather than by vendor
/// label (spec §6.16).
///
/// The four face buttons use compass-style names — [`South`](Self::South),
/// [`East`](Self::East), [`West`](Self::West), [`North`](Self::North) — which
/// map to A/B/X/Y (Xbox) or Cross/Circle/Square/Triangle (PlayStation) without
/// baking a vendor layout into the domain model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    /// The bottom face button (A / Cross).
    South,
    /// The right face button (B / Circle).
    East,
    /// The left face button (X / Square).
    West,
    /// The top face button (Y / Triangle).
    North,
    /// The up direction on the directional pad.
    DpadUp,
    /// The down direction on the directional pad.
    DpadDown,
    /// The left direction on the directional pad.
    DpadLeft,
    /// The right direction on the directional pad.
    DpadRight,
    /// The left shoulder bumper.
    LeftShoulder,
    /// The right shoulder bumper.
    RightShoulder,
    /// The left trigger, treated as a button (fully pressed).
    LeftTrigger,
    /// The right trigger, treated as a button (fully pressed).
    RightTrigger,
    /// The left analog stick pressed in.
    LeftStick,
    /// The right analog stick pressed in.
    RightStick,
    /// The Start / Menu button.
    Start,
    /// The Select / View / Back button.
    Select,
    /// The central Guide / Home button.
    Guide,
}

impl GamepadButton {
    /// Returns whether this is a directional-pad button.
    #[must_use]
    pub const fn is_dpad(&self) -> bool {
        matches!(
            self,
            Self::DpadUp | Self::DpadDown | Self::DpadLeft | Self::DpadRight
        )
    }

    /// Returns whether this is one of the four face buttons
    /// ([`South`](Self::South), [`East`](Self::East), [`West`](Self::West),
    /// [`North`](Self::North)).
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
