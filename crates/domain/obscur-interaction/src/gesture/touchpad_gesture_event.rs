use std::num::NonZeroUsize;

use obscur_geometry::Point;

use crate::event::EventTrust;

use super::{GesturePhase, TouchpadGestureDelta};

/// Native touchpad measurements before an application assigns an action.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchpadGestureEvent {
    /// Phase supplied by the backend.
    pub phase: GesturePhase,
    /// Gesture anchor in the coordinate space of the receiving API.
    pub position: Point,
    /// Reported contact count, when available.
    pub finger_count: Option<NonZeroUsize>,
    /// Measurements from this platform sample.
    pub delta: TouchpadGestureDelta,
    /// Whether the host vouches for the input.
    pub trust: EventTrust,
    /// Time in the host input clock.
    pub timestamp_micros: u64,
}
