//! Gesture recognition phases.

/// The lifecycle of a recognized input gesture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GesturePhase {
    /// The movement crossed the recognition threshold.
    Started,
    /// The recognized contact or scroll sequence moved.
    Updated,
    /// The gesture ended successfully.
    Completed,
    /// The sequence was interrupted or failed its completion condition.
    Cancelled,
}
