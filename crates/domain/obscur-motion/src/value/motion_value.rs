#![doc = include_str!("motion_value.doc/_module.md")]

use obscur_geometry::{Point, Rect, Size};

use super::Interpolatable;
use super::transform_interp::{self, Matrix2D};

#[doc = include_str!("motion_value.doc/MotionValue.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotionValue {
    #[doc = include_str!("motion_value.doc/Scalar.md")]
    Scalar(f32),
    #[doc = include_str!("motion_value.doc/Point.md")]
    Point(Point),
    #[doc = include_str!("motion_value.doc/Size.md")]
    Size(Size),
    #[doc = include_str!("motion_value.doc/Rect.md")]
    Rect(Rect),
    #[doc = include_str!("motion_value.doc/Transform.md")]
    Transform(Matrix2D),
    #[doc = include_str!("motion_value.doc/Color.md")]
    Color([f32; 4]),
}

impl MotionValue {
    #[doc = include_str!("motion_value.doc/scalar.md")]
    #[must_use]
    pub const fn scalar(value: f32) -> Self {
        Self::Scalar(value)
    }

    #[doc = include_str!("motion_value.doc/point.md")]
    #[must_use]
    pub const fn point(value: Point) -> Self {
        Self::Point(value)
    }

    #[doc = include_str!("motion_value.doc/size.md")]
    #[must_use]
    pub const fn size(value: Size) -> Self {
        Self::Size(value)
    }

    #[doc = include_str!("motion_value.doc/rect.md")]
    #[must_use]
    pub const fn rect(value: Rect) -> Self {
        Self::Rect(value)
    }

    #[doc = include_str!("motion_value.doc/transform.md")]
    #[must_use]
    pub const fn transform(matrix: Matrix2D) -> Self {
        Self::Transform(matrix)
    }

    #[doc = include_str!("motion_value.doc/color.md")]
    #[must_use]
    pub const fn color(rgba: [f32; 4]) -> Self {
        Self::Color(rgba)
    }

    #[doc = include_str!("motion_value.doc/as_scalar.md")]
    #[must_use]
    pub const fn as_scalar(&self) -> Option<f32> {
        match self {
            Self::Scalar(value) => Some(*value),
            _ => None,
        }
    }

    #[doc = include_str!("motion_value.doc/as_point.md")]
    #[must_use]
    pub const fn as_point(&self) -> Option<Point> {
        match self {
            Self::Point(value) => Some(*value),
            _ => None,
        }
    }

    #[doc = include_str!("motion_value.doc/as_size.md")]
    #[must_use]
    pub const fn as_size(&self) -> Option<Size> {
        match self {
            Self::Size(value) => Some(*value),
            _ => None,
        }
    }

    #[doc = include_str!("motion_value.doc/as_rect.md")]
    #[must_use]
    pub const fn as_rect(&self) -> Option<Rect> {
        match self {
            Self::Rect(value) => Some(*value),
            _ => None,
        }
    }

    #[doc = include_str!("motion_value.doc/as_transform.md")]
    #[must_use]
    pub const fn as_transform(&self) -> Option<Matrix2D> {
        match self {
            Self::Transform(value) => Some(*value),
            _ => None,
        }
    }

    #[doc = include_str!("motion_value.doc/as_color.md")]
    #[must_use]
    pub const fn as_color(&self) -> Option<[f32; 4]> {
        match self {
            Self::Color(value) => Some(*value),
            _ => None,
        }
    }

    #[doc = include_str!("motion_value.doc/kind_matches.md")]
    #[must_use]
    pub const fn kind_matches(&self, other: &MotionValue) -> bool {
        matches!(
            (self, other),
            (Self::Scalar(_), Self::Scalar(_))
                | (Self::Point(_), Self::Point(_))
                | (Self::Size(_), Self::Size(_))
                | (Self::Rect(_), Self::Rect(_))
                | (Self::Transform(_), Self::Transform(_))
                | (Self::Color(_), Self::Color(_))
        )
    }

    #[doc = include_str!("motion_value.doc/interpolate_checked.md")]
    #[must_use]
    pub fn interpolate_checked(&self, other: &MotionValue, t: f32) -> Option<MotionValue> {
        match (self, other) {
            (Self::Scalar(a), Self::Scalar(b)) => Some(Self::Scalar(a.interpolate(b, t))),
            (Self::Point(a), Self::Point(b)) => Some(Self::Point(a.interpolate(b, t))),
            (Self::Size(a), Self::Size(b)) => Some(Self::Size(a.interpolate(b, t))),
            (Self::Rect(a), Self::Rect(b)) => Some(Self::Rect(a.interpolate(b, t))),
            // Transforms do NOT blend component-wise: they interpolate by matrix
            // decomposition (CSS Transforms §14) so rotation/scale/skew stay
            // physical instead of shearing (see `transform_interp`).
            (Self::Transform(a), Self::Transform(b)) => Some(Self::Transform(
                transform_interp::interpolate_matrix(*a, *b, t),
            )),
            // Colors interpolate with PREMULTIPLIED alpha (css-color-4 §13.3): the
            // rgb channels are multiplied by alpha, all four components lerp, then
            // rgb is divided back out by the interpolated alpha. This keeps a
            // fade-to-transparent from muddying the visible hue toward the
            // transparent endpoint's channels.
            (Self::Color(a), Self::Color(b)) => Some(Self::Color(interpolate_color(*a, *b, t))),
            _ => None,
        }
    }
}

#[doc = include_str!("motion_value.doc/interpolate_color.md")]
fn interpolate_color(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let pa = [a[0] * a[3], a[1] * a[3], a[2] * a[3], a[3]];
    let pb = [b[0] * b[3], b[1] * b[3], b[2] * b[3], b[3]];
    let m = [
        pa[0].interpolate(&pb[0], t),
        pa[1].interpolate(&pb[1], t),
        pa[2].interpolate(&pb[2], t),
        pa[3].interpolate(&pb[3], t),
    ];
    let alpha = m[3];
    if alpha == 0.0 {
        [0.0, 0.0, 0.0, 0.0]
    } else {
        [m[0] / alpha, m[1] / alpha, m[2] / alpha, alpha]
    }
}

impl Interpolatable for MotionValue {
    #[doc = include_str!("motion_value.doc/interpolate.md")]
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        match self.interpolate_checked(other, t) {
            Some(value) => value,
            None => {
                if t < 0.5 {
                    *self
                } else {
                    *other
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_and_accessors() {
        assert_eq!(MotionValue::scalar(0.5).as_scalar(), Some(0.5));
        assert_eq!(MotionValue::scalar(0.5).as_point(), None);
        let p = Point::new(1.0, 2.0);
        assert_eq!(MotionValue::point(p).as_point(), Some(p));
        let s = Size::new(3.0, 4.0);
        assert_eq!(MotionValue::size(s).as_size(), Some(s));
        let r = Rect::new(p, s);
        assert_eq!(MotionValue::rect(r).as_rect(), Some(r));
    }

    #[test]
    fn kind_matches_only_same_variant() {
        assert!(MotionValue::scalar(0.0).kind_matches(&MotionValue::scalar(1.0)));
        assert!(!MotionValue::scalar(0.0).kind_matches(&MotionValue::point(Point::new(0.0, 0.0))));
    }

    #[test]
    fn interpolate_checked_some_on_match() {
        let a = MotionValue::scalar(0.0);
        let b = MotionValue::scalar(10.0);
        assert_eq!(
            a.interpolate_checked(&b, 0.5),
            Some(MotionValue::scalar(5.0))
        );

        let pa = MotionValue::point(Point::new(0.0, 0.0));
        let pb = MotionValue::point(Point::new(4.0, 8.0));
        assert_eq!(
            pa.interpolate_checked(&pb, 0.25),
            Some(MotionValue::point(Point::new(1.0, 2.0)))
        );
    }

    #[test]
    fn transform_constructor_and_accessor() {
        let m = [1.0, 0.0, 0.0, 1.0, 3.0, 4.0];
        assert_eq!(MotionValue::transform(m).as_transform(), Some(m));
        assert_eq!(MotionValue::scalar(1.0).as_transform(), None);
    }

    #[test]
    fn transform_interpolates_via_decomposition_not_component_wise() {
        // rotate(0°) → rotate(90°) at t=0.5 must be rotate(45°), scale preserved.
        let a = MotionValue::transform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
        let b = MotionValue::transform([0.0, 1.0, -1.0, 0.0, 0.0, 0.0]);
        let mid = a
            .interpolate_checked(&b, 0.5)
            .and_then(|v| v.as_transform())
            .expect("matching Transform variants blend");

        let s = 0.5_f32.sqrt(); // ≈ 0.7071
        for (got, want) in mid.iter().zip([s, s, -s, s, 0.0, 0.0]) {
            assert!((got - want).abs() <= 1e-4, "got {mid:?}");
        }
        // Scale is preserved at 1, unlike the naive lerp's ~0.707 shrink.
        assert!((mid[0].hypot(mid[1]) - 1.0).abs() <= 1e-4);
    }

    #[test]
    fn transform_kind_matches_only_transform() {
        let a = MotionValue::transform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
        let b = MotionValue::transform([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]);
        assert!(a.kind_matches(&b));
        assert!(!a.kind_matches(&MotionValue::scalar(1.0)));
    }

    #[test]
    fn interpolate_checked_none_on_mismatch() {
        let a = MotionValue::scalar(0.0);
        let b = MotionValue::size(Size::new(4.0, 4.0));
        assert_eq!(a.interpolate_checked(&b, 0.5), None);
    }

    #[test]
    fn trait_interpolate_blends_matching_variants() {
        let a = MotionValue::scalar(0.0);
        let b = MotionValue::scalar(10.0);
        assert_eq!(a.interpolate(&b, 0.5), MotionValue::scalar(5.0));
    }

    #[test]
    fn trait_interpolate_discrete_swap_on_mismatch() {
        let a = MotionValue::scalar(0.0);
        let b = MotionValue::size(Size::new(4.0, 4.0));
        // Below the midpoint the result stays `self`; at/after it switches to `other`.
        assert_eq!(a.interpolate(&b, 0.25), a);
        assert_eq!(a.interpolate(&b, 0.5), b);
        assert_eq!(a.interpolate(&b, 0.75), b);
    }
}
