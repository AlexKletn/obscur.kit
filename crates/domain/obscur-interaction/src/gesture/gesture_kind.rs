/// The kind of gesture a recognizer detects (spec §6.16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GestureKind {
    /// A single tap.
    Tap,
    /// Two taps in quick succession.
    DoubleTap,
    /// A press held past a duration threshold.
    LongPress,
    /// A one-or-more-finger drag.
    Pan,
    /// A quick directional flick.
    Swipe,
    /// A two-finger scale.
    Pinch,
    /// A two-finger rotation.
    Rotate,
}
