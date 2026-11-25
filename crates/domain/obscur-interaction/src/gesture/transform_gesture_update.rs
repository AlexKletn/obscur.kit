use std::num::NonZeroUsize;

use obscur_geometry::Point;

use super::{GesturePhase, GestureSource};

/// Simultaneous pinch and rotation measurements from a touchscreen or touchpad.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformGestureUpdate {
    /// Source of the gesture throughout this sequence.
    pub source: GestureSource,
    /// Number of contacts reported when the sequence began, if supplied.
    pub finger_count: Option<NonZeroUsize>,
    /// Lifecycle phase of this sample.
    pub phase: GesturePhase,
    /// The touchscreen contact midpoint or the initial touchpad cursor anchor, in logical coordinates.
    pub center: Point,
    /// Ratio to the initial distance between the contacts.
    pub scale: f32,
    /// Clockwise rotation from the initial pose, in radians.
    pub rotation: f32,
}
