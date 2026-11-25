//! Device-independent gesture feedback.

use super::{BackGesturePhase, GestureSource};

/// A semantic back gesture update independent of the input device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackGestureUpdate {
    /// Source of the gesture throughout this sequence.
    pub source: GestureSource,
    /// The current phase of the gesture.
    pub phase: BackGesturePhase,
    /// Rightward displacement in logical pixels.
    pub offset: f32,
}
