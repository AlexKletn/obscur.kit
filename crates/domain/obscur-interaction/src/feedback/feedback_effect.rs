/// Semantic cue selected by the component or host that owns the interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FeedbackEffect {
    /// Activation of a control.
    Click,
    /// Crossing a discrete step or threshold.
    Tick,
    /// Confirmation of a choice or action.
    Confirm,
    /// Rejection of a choice or action.
    Reject,
    /// Successful completion of an operation.
    Success,
    /// A warning that calls for attention.
    Warning,
    /// Failure of an operation.
    Error,
    /// Recognition of a long press.
    LongPress,
    /// Activation of a key on an on-screen keyboard.
    KeyboardTap,
    /// Crossing a discrete step while scrolling.
    ScrollTick,
}
