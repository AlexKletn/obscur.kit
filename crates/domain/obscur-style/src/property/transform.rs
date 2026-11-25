//! A style-local resolved 2D `transform` (spec §5.2).

#[doc = include_str!("../../docs/api/property/transform.doc/Transform.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// The resolved affine coefficients `[a, b, c, d, e, f]`.
    matrix: [f32; 6],
    /// Whether the authored value was `none` (the initial). A `none` transform has
    /// the identity matrix but does **not** establish a containing block; an
    /// authored transform does, even if its matrix is the identity.
    is_none: bool,
}

impl Transform {
    /// The identity transform (`matrix(1, 0, 0, 1, 0, 0)`), treated as an authored
    /// (containing-block-establishing) value. For the CSS `none` keyword use
    /// [`Transform::NONE`].
    pub const IDENTITY: Self = Self {
        matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        is_none: false,
    };

    /// The `transform: none` value — the identity matrix that does **not** establish
    /// a containing block for positioned descendants.
    pub const NONE: Self = Self {
        matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        is_none: true,
    };

    /// Creates a transform from its resolved affine coefficients
    /// `[a, b, c, d, e, f]`. The result is an authored transform (it establishes a
    /// containing block); the CSS `none` keyword is [`Transform::NONE`] instead.
    #[must_use]
    pub const fn new(matrix: [f32; 6]) -> Self {
        Self {
            matrix,
            is_none: false,
        }
    }

    /// Returns the resolved affine coefficients `[a, b, c, d, e, f]`.
    #[must_use]
    pub const fn matrix(&self) -> [f32; 6] {
        self.matrix
    }

    /// Whether this transform is (exactly) the identity matrix. Note that both
    /// [`Transform::NONE`] and an authored `translateX(0px)` are identity here; use
    /// [`establishes_containing_block`](Self::establishes_containing_block) to tell
    /// them apart.
    #[must_use]
    pub fn is_identity(&self) -> bool {
        self.matrix == Self::IDENTITY.matrix
    }

    /// Whether this is the `none` value.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.is_none
    }

    /// Whether this transform establishes a containing block for its fixed- and
    /// absolute-positioned descendants (css-transforms-1 §3: "any value other than
    /// `none`"). True for every authored transform, including one whose matrix
    /// resolves to the identity; false only for [`Transform::NONE`].
    #[must_use]
    pub const fn establishes_containing_block(&self) -> bool {
        !self.is_none
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
