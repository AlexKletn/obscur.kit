#![doc = include_str!("pointer_tilt.doc/_module.md")]

#[doc = include_str!("pointer_tilt.doc/PointerTilt.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PointerTilt {
    #[doc = include_str!("pointer_tilt.doc/x_degrees.md")]
    pub x_degrees: f32,
    #[doc = include_str!("pointer_tilt.doc/y_degrees.md")]
    pub y_degrees: f32,
}

impl PointerTilt {
    #[doc = include_str!("pointer_tilt.doc/UPRIGHT.md")]
    pub const UPRIGHT: Self = Self {
        x_degrees: 0.0,
        y_degrees: 0.0,
    };

    #[doc = include_str!("pointer_tilt.doc/new.md")]
    #[must_use]
    pub fn new(x_degrees: f32, y_degrees: f32) -> Self {
        Self {
            x_degrees: x_degrees.clamp(-90.0, 90.0),
            y_degrees: y_degrees.clamp(-90.0, 90.0),
        }
    }

    #[doc = include_str!("pointer_tilt.doc/is_upright.md")]
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
