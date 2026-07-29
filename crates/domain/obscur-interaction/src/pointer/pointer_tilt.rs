//! The angle a stylus is held at, per axis (spec §2.4).

/// §2.4's pointer **tilt**: the angle between the pointer's axis and the surface
/// normal, in degrees, resolved per plane.
///
/// §2.4 lists tilt as a required pointer fact, alongside pressure and rotation,
/// because a pen is not a point: a drawing surface needs to know which way the
/// barrel leans to shape a stroke, and only the device can say.
///
/// Both angles are in `-90..=90` degrees, matching the Pointer Events convention:
/// [`x_degrees`](Self::x_degrees) is the tilt in the plane containing the Y axis
/// (a lean left/right, negative toward the left), and
/// [`y_degrees`](Self::y_degrees) is the tilt in the plane containing the X axis
/// (a lean toward/away from the user, negative toward the top). A device held
/// perpendicular to the surface, and every device that cannot report tilt at all,
/// reports [`UPRIGHT`](Self::UPRIGHT) — zero on both axes.
///
/// A mouse or a touchscreen contact therefore reports `UPRIGHT` too. That is a
/// deliberate flattening rather than a missing value: §2.4 requires the field on
/// every pointer event, and "no lean" is the honest reading of a device with no
/// barrel.
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

    /// Creates a tilt, clamping both angles into `-90..=90` degrees.
    ///
    /// Clamping rather than trusting the caller: a platform that reports tilt in
    /// a different unit, or an uninitialised field, would otherwise hand an
    /// application an angle no physical stylus can be at.
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
