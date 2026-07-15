//! 2D affine transform.

use crate::primitive::Point;

/// A 2D affine transform stored as the six coefficients of a 2x3 matrix.
///
/// Maps a point `p` to `(a*x + c*y + tx, b*x + d*y + ty)`.
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

    /// Composes two transforms: `self.multiply(rhs)` is the transform that
    /// applies `rhs` **first**, then `self` (standard matrix product
    /// `self · rhs`).
    #[must_use]
    pub fn multiply(self, rhs: Self) -> Self {
        let a = self.coeffs;
        let b = rhs.coeffs;
        Self {
            coeffs: [
                a[0] * b[0] + a[2] * b[1],
                a[1] * b[0] + a[3] * b[1],
                a[0] * b[2] + a[2] * b[3],
                a[1] * b[2] + a[3] * b[3],
                a[0] * b[4] + a[2] * b[5] + a[4],
                a[1] * b[4] + a[3] * b[5] + a[5],
            ],
        }
    }

    /// Whether this transform is (exactly) the identity.
    #[must_use]
    pub fn is_identity(self) -> bool {
        self.coeffs == Self::IDENTITY.coeffs
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
}
