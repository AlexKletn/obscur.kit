//! A style-local resolved 2D `transform` (spec §5.2).

/// A CSS 2D `transform` resolved to a single 2x3 affine matrix.
///
/// The authored `<transform-function>+` list (`translate`, `scale`, `rotate`,
/// `skew`, `matrix`) is composed **left-to-right** by the CSS lowering into one
/// affine, stored here as the six coefficients `[a, b, c, d, e, f]` of a
/// `matrix(a, b, c, d, e, f)` — the same layout
/// [`obscur_geometry::AffineTransform`] uses. A point `(x, y)` maps to
/// `(a·x + c·y + e, b·x + d·y + f)`.
///
/// `transform: none` lowers to no value at all (an absent property), so a stored
/// `Transform` is always a real, authored transform. The scene coordinator wraps
/// the affine with its transform origin (`translate(origin) · matrix ·
/// translate(-origin)`) when it brackets the element's subtree.
///
/// Because the coefficients are `f32`, this derives [`PartialEq`] but not
/// [`Eq`]/[`Hash`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// The resolved affine coefficients `[a, b, c, d, e, f]`.
    matrix: [f32; 6],
}

impl Transform {
    /// The identity transform (`matrix(1, 0, 0, 1, 0, 0)`).
    pub const IDENTITY: Self = Self {
        matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    /// Creates a transform from its resolved affine coefficients
    /// `[a, b, c, d, e, f]`.
    #[must_use]
    pub const fn new(matrix: [f32; 6]) -> Self {
        Self { matrix }
    }

    /// Returns the resolved affine coefficients `[a, b, c, d, e, f]`.
    #[must_use]
    pub const fn matrix(&self) -> [f32; 6] {
        self.matrix
    }

    /// Whether this transform is (exactly) the identity.
    #[must_use]
    pub fn is_identity(&self) -> bool {
        self.matrix == Self::IDENTITY.matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_and_returns_matrix() {
        let t = Transform::new([2.0, 0.0, 0.0, 2.0, 5.0, 6.0]);
        assert_eq!(t.matrix(), [2.0, 0.0, 0.0, 2.0, 5.0, 6.0]);
        assert!(!t.is_identity());
    }

    #[test]
    fn identity_is_identity() {
        assert!(Transform::IDENTITY.is_identity());
        assert_eq!(Transform::IDENTITY.matrix(), [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }
}
