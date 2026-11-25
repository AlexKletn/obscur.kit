/// Contact phase of a continuous scroll sequence.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WheelPhase {
    /// The source did not report a continuous gesture phase.
    #[default]
    Unspecified,
    /// The continuous scroll sequence began.
    Started,
    /// The continuous sequence delivered another scroll delta.
    Moved,
    /// The source finished the continuous sequence.
    Ended,
    /// The source interrupted the continuous sequence.
    Cancelled,
}
