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
