use std::num::NonZeroUsize;

use super::{GesturePhase, GestureSource, SwipeDirection};

/// A sample of a recognized directional swipe.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwipeUpdate {
    /// Source of the gesture throughout this sequence.
    pub source: GestureSource,
    /// Lifecycle phase of this sample.
    pub phase: GesturePhase,
    /// Direction locked when the swipe started.
    pub direction: SwipeDirection,
    /// Displacement along the locked direction in logical pixels.
    pub offset: f32,
    /// Number of contacts in the recognized swipe, including in its final update. Phased wheel input and native events without a reported count return `None`.
    pub finger_count: Option<NonZeroUsize>,
}
