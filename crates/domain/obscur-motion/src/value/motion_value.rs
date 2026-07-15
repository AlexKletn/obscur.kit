//! The `MotionValue` dynamic animatable value (spec §4.17).

use obscur_geometry::{Point, Rect, Size};

use super::Interpolatable;
use super::transform_interp::{self, Matrix2D};

/// A concrete presentation value the motion engine can animate (spec §4.17).
///
/// `MotionValue` is the dynamic, type-erased carrier for the engine's *arbitrary
/// typed interpolation*: a single enum spanning the supported presentation value
/// kinds so timelines, tracks and drivers can move any of them through one code
/// path. [`Scalar`](Self::Scalar) covers every 1-D quantity (opacity, rotation,
/// a scale factor, a single translation axis); the geometric variants carry
/// whole points, sizes and rects for transforms and layout-affecting animation.
///
/// Two `MotionValue`s only blend when they hold the *same* variant. Prefer
/// [`interpolate_checked`](Self::interpolate_checked), which reports a variant
/// mismatch as `None`; the [`Interpolatable`] impl instead degrades gracefully
/// to a discrete swap (see its documentation) because it must return `Self`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotionValue {
    /// A 1-D scalar: opacity, rotation, scale, a single translation axis, etc.
    Scalar(f32),
    /// A 2-D point, e.g. a translation or an anchor position.
    Point(Point),
    /// A 2-D size, e.g. an animated width/height.
    Size(Size),
    /// An axis-aligned rectangle, e.g. an animated layout frame.
    Rect(Rect),
    /// A 2D `transform`: the six affine coefficients `[a, b, c, d, e, f]` of a
    /// `matrix(a, b, c, d, e, f)` (the lowered `<transform-function>+` list).
    ///
    /// Unlike the other variants this does **not** blend component-wise: it
    /// interpolates by matrix decomposition (CSS Transforms §14), so a
    /// `rotate(0°) → rotate(90°)` transition rotates through `rotate(45°)`
    /// instead of shearing and shrinking the box. See
    /// [`interpolate_checked`](Self::interpolate_checked).
    Transform(Matrix2D),
}

impl MotionValue {
    /// Wraps a 1-D scalar (opacity, rotation, any single dimension).
    #[must_use]
    pub const fn scalar(value: f32) -> Self {
        Self::Scalar(value)
    }

    /// Wraps a [`Point`].
    #[must_use]
    pub const fn point(value: Point) -> Self {
        Self::Point(value)
    }

    /// Wraps a [`Size`].
    #[must_use]
    pub const fn size(value: Size) -> Self {
        Self::Size(value)
    }

    /// Wraps a [`Rect`].
    #[must_use]
    pub const fn rect(value: Rect) -> Self {
        Self::Rect(value)
    }

    /// Wraps a 2D `transform` affine `[a, b, c, d, e, f]`.
    #[must_use]
    pub const fn transform(matrix: Matrix2D) -> Self {
        Self::Transform(matrix)
    }

    /// Returns the wrapped scalar, or `None` if this is not a
    /// [`Scalar`](Self::Scalar).
    #[must_use]
    pub const fn as_scalar(&self) -> Option<f32> {
        match self {
            Self::Scalar(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the wrapped point, or `None` if this is not a
    /// [`Point`](Self::Point).
    #[must_use]
    pub const fn as_point(&self) -> Option<Point> {
        match self {
            Self::Point(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the wrapped size, or `None` if this is not a
    /// [`Size`](Self::Size).
    #[must_use]
    pub const fn as_size(&self) -> Option<Size> {
        match self {
            Self::Size(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the wrapped rect, or `None` if this is not a
    /// [`Rect`](Self::Rect).
    #[must_use]
    pub const fn as_rect(&self) -> Option<Rect> {
        match self {
            Self::Rect(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the wrapped transform affine `[a, b, c, d, e, f]`, or `None` if
    /// this is not a [`Transform`](Self::Transform).
    #[must_use]
    pub const fn as_transform(&self) -> Option<Matrix2D> {
        match self {
            Self::Transform(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns whether `self` and `other` hold the same variant (and can
    /// therefore be blended by [`interpolate_checked`](Self::interpolate_checked)).
    #[must_use]
    pub const fn kind_matches(&self, other: &MotionValue) -> bool {
        matches!(
            (self, other),
            (Self::Scalar(_), Self::Scalar(_))
                | (Self::Point(_), Self::Point(_))
                | (Self::Size(_), Self::Size(_))
                | (Self::Rect(_), Self::Rect(_))
                | (Self::Transform(_), Self::Transform(_))
        )
    }

    /// Blends toward `other` component-wise, returning `None` on a variant
    /// mismatch (spec §4.17).
    ///
    /// `t` follows the [`Interpolatable`] contract: normalized progress in
    /// `[0, 1]`, not clamped. When both operands hold the same variant the
    /// inner value is interpolated and wrapped back up in `Some`; when the
    /// variants differ the two endpoints are type-incompatible and `None` is
    /// returned rather than a nonsensical blend.
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
            _ => None,
        }
    }
}

impl Interpolatable for MotionValue {
    /// Blends toward `other`, degrading gracefully on a variant mismatch.
    ///
    /// When both operands hold the same variant this delegates to
    /// [`interpolate_checked`](MotionValue::interpolate_checked) and blends the
    /// inner value component-wise. Because this trait method must return `Self`
    /// (not `Option`), a *mismatch* cannot be signalled; instead it performs a
    /// discrete swap — `self` while `t < 0.5`, otherwise `other` — so the
    /// animation still resolves to a valid endpoint. Callers that need to
    /// detect incompatibility should use `interpolate_checked` or
    /// [`InterpolationResult`](super::InterpolationResult) instead.
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
