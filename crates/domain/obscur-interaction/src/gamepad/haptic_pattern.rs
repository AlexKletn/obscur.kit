/// The shape of a haptic feedback effect (spec §6.16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HapticPattern {
    /// A single sharp click.
    Click,
    /// Two clicks in quick succession.
    DoubleClick,
    /// A light, brief tick.
    Tick,
    /// A sustained buzz for the request's duration.
    Buzz,
    /// A custom, platform-defined effect.
    Custom,
}
