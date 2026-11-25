use super::GesturePhase;

/// Predictive Back progress from the window host after the system claimed the gesture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemBackEvent {
    /// Start, progress, completion or cancellation of the system gesture.
    pub phase: GesturePhase,
    /// Normalized transition progress supplied by the system. The runtime clamps finite values to the unit interval.
    pub progress: f32,
}
