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
