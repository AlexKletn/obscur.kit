//! 2D affine transform.

use crate::primitive::{Point, Rect, Size};

#[doc = include_str!("../../docs/api/transform/affine_transform.doc/AffineTransform.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AffineTransform {
    /// Row-major coefficients `[a, b, c, d, tx, ty]`.
    pub coeffs: [f32; 6],
}

impl AffineTransform {
    /// The identity transform.
    pub const IDENTITY: Self = Self {
        coeffs: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    /// A pure translation.
    #[must_use]
    pub const fn translate(tx: f32, ty: f32) -> Self {
        Self {
            coeffs: [1.0, 0.0, 0.0, 1.0, tx, ty],
        }
    }

    /// An affine from the six coefficients of a CSS `matrix(a, b, c, d, e, f)`
    /// (which use the identical `[a, b, c, d, tx, ty]` layout).
    // The six single-letter names are the canonical CSS `matrix()` coefficients.
    #[allow(clippy::many_single_char_names)]
    #[must_use]
    pub const fn from_matrix(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self {
            coeffs: [a, b, c, d, e, f],
        }
    }

    /// A pure scale `(sx, sy)` about the origin.
    #[must_use]
    pub const fn scale(sx: f32, sy: f32) -> Self {
        Self {
            coeffs: [sx, 0.0, 0.0, sy, 0.0, 0.0],
        }
    }

    /// A rotation by `radians` (clockwise in the y-down screen space).
    #[must_use]
    pub fn rotate(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self {
            coeffs: [cos, sin, -sin, cos, 0.0, 0.0],
        }
    }

    /// A skew by the two angles (in radians): `ax` shears x by the point's y,
    /// `ay` shears y by the point's x (CSS `skew(ax, ay)`).
    #[must_use]
    pub fn skew(ax: f32, ay: f32) -> Self {
        Self {
            coeffs: [1.0, ay.tan(), ax.tan(), 1.0, 0.0, 0.0],
        }
    }

    #[doc = include_str!("../../docs/api/transform/affine_transform.doc/multiply.md")]
    #[must_use]
    pub fn multiply(self, rhs: Self) -> Self {
        let a = self.coeffs;
        let b = rhs.coeffs;
        let coeffs = [
            a[0] * b[0] + a[2] * b[1],
            a[1] * b[0] + a[3] * b[1],
            a[0] * b[2] + a[2] * b[3],
            a[1] * b[2] + a[3] * b[3],
            a[0] * b[4] + a[2] * b[5] + a[4],
            a[1] * b[4] + a[3] * b[5] + a[5],
        ];
        if coeffs.iter().all(|value| value.is_finite()) {
            return Self { coeffs };
        }
        // Recompute before narrowing: clamping intermediate products would
        // destroy cancellations and leave NaN from opposite infinities.
        let a = a.map(f64::from);
        let b = b.map(f64::from);
        Self {
            coeffs: [
                a[0] * b[0] + a[2] * b[1],
                a[1] * b[0] + a[3] * b[1],
                a[0] * b[2] + a[2] * b[3],
                a[1] * b[2] + a[3] * b[3],
                a[0] * b[4] + a[2] * b[5] + a[4],
                a[1] * b[4] + a[3] * b[5] + a[5],
            ]
            .map(crate::saturating_f32),
        }
    }

    /// Whether this transform is (exactly) the identity.
    #[must_use]
    pub fn is_identity(self) -> bool {
        self.coeffs == Self::IDENTITY.coeffs
    }

    #[doc = include_str!("../../docs/api/transform/affine_transform.doc/inverse.md")]
    // The six single-letter names are the canonical CSS `matrix()` coefficients.
    #[allow(clippy::many_single_char_names)]
    #[must_use]
    pub fn inverse(self) -> Option<Self> {
        let [a, b, c, d, e, f] = self.coeffs;
        let det = a * d - b * c;
        // A zero (or non-finite) determinant is a collapsed plane: no inverse
        // exists, and dividing would hand back infinities that then poison every
        // point mapped through them.
        if det == 0.0 || !det.is_finite() {
            return None;
        }
        let inv = 1.0 / det;
        let out = Self {
            coeffs: [
                d * inv,
                -b * inv,
                -c * inv,
                a * inv,
                (c * f - d * e) * inv,
                (b * e - a * f) * inv,
            ],
        };
        // A determinant that is finite but denormal-small can still overflow the
        // reciprocal; only a wholly finite result is safe to hand out.
        out.coeffs.iter().all(|v| v.is_finite()).then_some(out)
    }

    /// Applies this transform to a point.
    #[must_use]
    pub fn apply(self, p: Point) -> Point {
        let m = self.coeffs;
        Point::new(
            m[0] * p.x + m[2] * p.y + m[4],
            m[1] * p.x + m[3] * p.y + m[5],
        )
    }

    #[doc = include_str!("../../docs/api/transform/affine_transform.doc/map_rect.md")]
    #[must_use]
    pub fn map_rect(self, rect: Rect) -> Rect {
        let (l, t, r, b) = (rect.origin.x, rect.origin.y, rect.right(), rect.bottom());
        let corners = [
            self.apply(Point::new(l, t)),
            self.apply(Point::new(r, t)),
            self.apply(Point::new(r, b)),
            self.apply(Point::new(l, b)),
        ];
        let min_x = corners.iter().fold(f32::INFINITY, |m, p| m.min(p.x));
        let min_y = corners.iter().fold(f32::INFINITY, |m, p| m.min(p.y));
        let max_x = corners.iter().fold(f32::NEG_INFINITY, |m, p| m.max(p.x));
        let max_y = corners.iter().fold(f32::NEG_INFINITY, |m, p| m.max(p.y));
        Rect::new(
            Point::new(min_x, min_y),
            Size::new(max_x - min_x, max_y - min_y),
        )
    }
}

impl Default for AffineTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: [f32; 6], b: [f32; 6]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5)
    }

    #[test]
    fn scale_and_translate_coeffs() {
        assert_eq!(
            AffineTransform::scale(2.0, 3.0).coeffs,
            [2.0, 0.0, 0.0, 3.0, 0.0, 0.0]
        );
        assert_eq!(
            AffineTransform::translate(5.0, 6.0).coeffs,
            [1.0, 0.0, 0.0, 1.0, 5.0, 6.0]
        );
    }

    #[test]
    fn rotate_ninety_degrees_maps_axes() {
        let r = AffineTransform::rotate(std::f32::consts::FRAC_PI_2);
        assert!(near(r.coeffs, [0.0, 1.0, -1.0, 0.0, 0.0, 0.0]));
        // (1, 0) rotates to (0, 1) in y-down space.
        let p = r.apply(Point::new(1.0, 0.0));
        assert!((p.x).abs() < 1e-5 && (p.y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn multiply_composes_right_first() {
        // translate then scale: scale.multiply(translate) applies translate first.
        let m = AffineTransform::scale(2.0, 2.0).multiply(AffineTransform::translate(1.0, 0.0));
        // point (0,0) -> translate (1,0) -> scale (2,0).
        let p = m.apply(Point::new(0.0, 0.0));
        assert!((p.x - 2.0).abs() < 1e-5 && p.y.abs() < 1e-5);
    }

    #[test]
    fn identity_detection() {
        assert!(AffineTransform::IDENTITY.is_identity());
        assert!(!AffineTransform::scale(2.0, 1.0).is_identity());
    }

    #[test]
    fn a_translation_maps_a_rect_exactly() {
        let r = Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 40.0));
        let mapped = AffineTransform::translate(10.0, -5.0).map_rect(r);
        assert_eq!(mapped, Rect::new(Point::new(10.0, -5.0), r.size));
    }

    #[test]
    fn a_quarter_turn_maps_a_rect_to_its_swapped_bounding_box() {
        let r = Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 40.0));
        let mapped = AffineTransform::rotate(std::f32::consts::FRAC_PI_2).map_rect(r);
        assert!((mapped.origin.x - -40.0).abs() < 1e-3, "{mapped:?}");
        assert!(mapped.origin.y.abs() < 1e-3, "{mapped:?}");
        assert!((mapped.size.width - 40.0).abs() < 1e-3, "{mapped:?}");
        assert!((mapped.size.height - 100.0).abs() < 1e-3, "{mapped:?}");
    }

    #[test]
    fn an_inverse_undoes_the_transform_it_came_from() {
        let t = AffineTransform::translate(50.0, 20.0)
            .multiply(AffineTransform::rotate(std::f32::consts::FRAC_PI_4))
            .multiply(AffineTransform::translate(-50.0, -20.0))
            .multiply(AffineTransform::scale(2.0, 0.5));
        let inv = t
            .inverse()
            .expect("a rotate/scale/translate chain is invertible");
        for p in [
            Point::new(0.0, 0.0),
            Point::new(3.0, -7.5),
            Point::new(120.0, 40.0),
        ] {
            let round_trip = inv.apply(t.apply(p));
            assert!(
                (round_trip.x - p.x).abs() < 1e-3 && (round_trip.y - p.y).abs() < 1e-3,
                "{p:?} round-tripped to {round_trip:?}"
            );
        }
    }

    #[test]
    fn the_identity_inverts_to_itself() {
        assert_eq!(
            AffineTransform::IDENTITY.inverse(),
            Some(AffineTransform::IDENTITY)
        );
    }

    #[test]
    fn a_singular_matrix_has_no_inverse_and_does_not_panic() {
        // Every one of these collapses the plane onto a line or a point, so no
        // inverse exists — the caller must be handed `None`, not an infinity.
        assert_eq!(AffineTransform::scale(0.0, 1.0).inverse(), None);
        assert_eq!(AffineTransform::scale(1.0, 0.0).inverse(), None);
        assert_eq!(AffineTransform::scale(0.0, 0.0).inverse(), None);
        // Both columns parallel: determinant zero with no zero coefficient.
        assert_eq!(
            AffineTransform::from_matrix(2.0, 4.0, 1.0, 2.0, 7.0, 9.0).inverse(),
            None
        );
    }

    #[test]
    fn a_non_finite_matrix_has_no_inverse() {
        assert_eq!(
            AffineTransform::from_matrix(f32::NAN, 0.0, 0.0, 1.0, 0.0, 0.0).inverse(),
            None
        );
        assert_eq!(
            AffineTransform::from_matrix(f32::INFINITY, 0.0, 0.0, 1.0, 0.0, 0.0).inverse(),
            None
        );
        // Every coefficient and the determinant are finite, but the inverted
        // translation overflows `f32` — an unusable result, so also `None`.
        assert_eq!(
            AffineTransform::from_matrix(1e-4, 0.0, 0.0, 1e-4, 1e35, 0.0).inverse(),
            None
        );
    }
}
