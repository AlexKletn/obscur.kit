//! The movement a pointer reported since its previous event (spec §2.4).

/// §2.4's pointer **delta**: how far this pointer moved since its own previous
/// event, in logical pixels.
///
/// §2.4 requires a pointer event to carry *"logical position and delta"* — two
/// facts, not one. The position is where the pointer is; the delta is how it got
/// there, and it is not derivable from two successive positions in every case a
/// handler cares about. A relative device (a trackpad, a captured mouse) can
/// report motion the position does not show once the cursor is clamped at the edge
/// of the screen, and a coalesced move reports the *sum* of several motions at a
/// single position.
///
/// # Why this is not `obscur_geometry::Vector`
///
/// Because `Vector` is still a skeleton stub (`pub(crate) struct Vector;`) with no
/// contract implemented. Modeling the delta locally keeps §2.4's field honest
/// today rather than blocking it on a foundation type; when `Vector` lands this
/// becomes a type alias or a `From` pair, and no caller has to change shape
/// because the accessors already read `x` / `y`.
///
/// A pointer that did not move — a press, a release, an enter — carries
/// [`ZERO`](Self::ZERO), which is the truth rather than a placeholder: nothing
/// moved.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PointerDelta {
    /// Horizontal movement in logical pixels, positive to the right.
    pub x: f32,
    /// Vertical movement in logical pixels, positive downward.
    pub y: f32,
}

impl PointerDelta {
    /// No movement — a press, a release, or an enter.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Creates a delta of `x` horizontal and `y` vertical logical pixels.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns whether this delta reports no movement at all.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    /// Returns the straight-line distance this delta covers, in logical pixels.
    #[must_use]
    pub fn magnitude(self) -> f32 {
        self.x.hypot(self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_the_default_and_reports_no_movement() {
        assert_eq!(PointerDelta::default(), PointerDelta::ZERO);
        assert!(PointerDelta::ZERO.is_zero());
        assert_eq!(PointerDelta::ZERO.magnitude(), 0.0);
    }

    #[test]
    fn magnitude_is_the_hypotenuse() {
        let delta = PointerDelta::new(3.0, -4.0);
        assert!(!delta.is_zero());
        assert_eq!(delta.magnitude(), 5.0);
    }
}
