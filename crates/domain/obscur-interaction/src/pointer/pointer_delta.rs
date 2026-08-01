#![doc = include_str!("pointer_delta.doc/_module.md")]

#[doc = include_str!("pointer_delta.doc/PointerDelta.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PointerDelta {
    #[doc = include_str!("pointer_delta.doc/x.md")]
    pub x: f32,
    #[doc = include_str!("pointer_delta.doc/y.md")]
    pub y: f32,
}

impl PointerDelta {
    #[doc = include_str!("pointer_delta.doc/ZERO.md")]
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[doc = include_str!("pointer_delta.doc/new.md")]
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[doc = include_str!("pointer_delta.doc/is_zero.md")]
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    #[doc = include_str!("pointer_delta.doc/magnitude.md")]
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
