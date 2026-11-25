//! The angle a stylus is held at, per axis (spec §2.4).

#[doc = include_str!("../../docs/api/pointer/pointer_tilt.doc/PointerTilt.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PointerTilt {
    /// Tilt in the plane containing the Y axis, in `-90..=90` degrees; negative
    /// leans left.
    pub x_degrees: f32,
    /// Tilt in the plane containing the X axis, in `-90..=90` degrees; negative
    /// leans toward the top of the surface.
    pub y_degrees: f32,
}

impl PointerTilt {
    /// A device held perpendicular to the surface, and the reading for any device
    /// that cannot report tilt.
    pub const UPRIGHT: Self = Self {
        x_degrees: 0.0,
        y_degrees: 0.0,
    };

    #[doc = include_str!("../../docs/api/pointer/pointer_tilt.doc/new.md")]
    #[must_use]
    pub fn new(x_degrees: f32, y_degrees: f32) -> Self {
        Self {
            x_degrees: x_degrees.clamp(-90.0, 90.0),
            y_degrees: y_degrees.clamp(-90.0, 90.0),
        }
    }

    /// Returns whether the device is perpendicular to the surface (no lean on
    /// either axis).
    #[must_use]
    pub fn is_upright(self) -> bool {
        self.x_degrees == 0.0 && self.y_degrees == 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upright_is_the_default() {
        assert_eq!(PointerTilt::default(), PointerTilt::UPRIGHT);
        assert!(PointerTilt::UPRIGHT.is_upright());
    }

    #[test]
    fn angles_are_clamped_to_the_physical_range() {
        let tilt = PointerTilt::new(-120.0, 200.0);
        assert_eq!(tilt.x_degrees, -90.0);
        assert_eq!(tilt.y_degrees, 90.0);
        assert!(!tilt.is_upright());
    }

    #[test]
    fn an_in_range_tilt_is_kept_verbatim() {
        let tilt = PointerTilt::new(-30.0, 45.0);
        assert_eq!(tilt.x_degrees, -30.0);
        assert_eq!(tilt.y_degrees, 45.0);
    }
}
