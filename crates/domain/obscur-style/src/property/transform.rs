#![doc = include_str!("transform.doc/_module.md")]

#[doc = include_str!("transform.doc/Transform.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    #[doc = include_str!("transform.doc/matrix.md")]
    matrix: [f32; 6],
    #[doc = include_str!("transform.doc/is_none.md")]
    is_none: bool,
}

impl Transform {
    #[doc = include_str!("transform.doc/IDENTITY.md")]
    pub const IDENTITY: Self = Self {
        matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        is_none: false,
    };

    #[doc = include_str!("transform.doc/NONE.md")]
    pub const NONE: Self = Self {
        matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        is_none: true,
    };

    #[doc = include_str!("transform.doc/new.md")]
    #[must_use]
    pub const fn new(matrix: [f32; 6]) -> Self {
        Self {
            matrix,
            is_none: false,
        }
    }

    #[doc = include_str!("transform.doc/matrix_2.md")]
    #[must_use]
    pub const fn matrix(&self) -> [f32; 6] {
        self.matrix
    }

    #[doc = include_str!("transform.doc/is_identity.md")]
    #[must_use]
    pub fn is_identity(&self) -> bool {
        self.matrix == Self::IDENTITY.matrix
    }

    #[doc = include_str!("transform.doc/is_none_2.md")]
    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.is_none
    }

    #[doc = include_str!("transform.doc/establishes_containing_block.md")]
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
