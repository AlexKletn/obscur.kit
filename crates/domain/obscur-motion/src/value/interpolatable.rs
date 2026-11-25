//! The `Interpolatable` trait: arbitrary typed interpolation (spec §4.17).

use obscur_geometry::{Point, Rect, Size};

#[doc = include_str!("../../docs/api/value/interpolatable.doc/lerp_f32.md")]
#[must_use]
pub const fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[doc = include_str!("../../docs/api/value/interpolatable.doc/Interpolatable.md")]
pub trait Interpolatable {
    #[doc = include_str!("../../docs/api/value/interpolatable.doc/interpolate.md")]
    #[must_use]
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

impl Interpolatable for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        lerp_f32(*self, *other, t)
    }
}

impl Interpolatable for Point {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Point::new(lerp_f32(self.x, other.x, t), lerp_f32(self.y, other.y, t))
    }
}

impl Interpolatable for Size {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Size::new(
            lerp_f32(self.width, other.width, t),
            lerp_f32(self.height, other.height, t),
        )
    }
}

impl Interpolatable for Rect {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Rect::new(
            self.origin.interpolate(&other.origin, t),
            self.size.interpolate(&other.size, t),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_lerp_endpoints_and_midpoint() {
        assert_eq!(lerp_f32(2.0, 6.0, 0.0), 2.0);
        assert_eq!(lerp_f32(2.0, 6.0, 0.5), 4.0);
        assert_eq!(lerp_f32(2.0, 6.0, 1.0), 6.0);
    }

    #[test]
    fn f32_lerp_overshoot_is_not_clamped() {
        // t == 1.5 extrapolates past `other` rather than saturating at it.
        assert_eq!(lerp_f32(2.0, 6.0, 1.5), 8.0);
        assert_eq!(lerp_f32(2.0, 6.0, -0.5), 0.0);
    }

    #[test]
    fn f32_trait_matches_free_fn() {
        assert_eq!(2.0_f32.interpolate(&6.0, 0.25), 3.0);
    }

    #[test]
    fn point_lerp_is_component_wise() {
        let a = Point::new(0.0, 10.0);
        let b = Point::new(4.0, 20.0);
        assert_eq!(a.interpolate(&b, 0.5), Point::new(2.0, 15.0));
    }

    #[test]
    fn size_lerp_is_component_wise() {
        let a = Size::new(0.0, 8.0);
        let b = Size::new(4.0, 16.0);
        assert_eq!(a.interpolate(&b, 0.25), Size::new(1.0, 10.0));
    }

    #[test]
    fn rect_lerp_is_component_wise() {
        let a = Rect::new(Point::new(0.0, 0.0), Size::new(0.0, 0.0));
        let b = Rect::new(Point::new(2.0, 4.0), Size::new(8.0, 12.0));
        assert_eq!(
            a.interpolate(&b, 0.5),
            Rect::new(Point::new(1.0, 2.0), Size::new(4.0, 6.0))
        );
    }
}
