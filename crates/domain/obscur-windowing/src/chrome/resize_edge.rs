#[doc = include_str!("../../docs/api/chrome/resize_region.doc/ResizeEdge.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizeEdge {
    /// The top edge.
    Top,
    /// The bottom edge.
    Bottom,
    /// The left edge.
    Left,
    /// The right edge.
    Right,
    /// The top-left corner.
    TopLeft,
    /// The top-right corner.
    TopRight,
    /// The bottom-left corner.
    BottomLeft,
    /// The bottom-right corner.
    BottomRight,
}

impl ResizeEdge {
    /// Returns whether this edge is a corner (resizes in both axes).
    #[must_use]
    pub const fn is_corner(self) -> bool {
        matches!(
            self,
            Self::TopLeft | Self::TopRight | Self::BottomLeft | Self::BottomRight
        )
    }

    /// Returns whether resizing this edge changes the window's width.
    #[must_use]
    pub const fn is_horizontal(self) -> bool {
        matches!(
            self,
            Self::Left
                | Self::Right
                | Self::TopLeft
                | Self::TopRight
                | Self::BottomLeft
                | Self::BottomRight
        )
    }

    /// Returns whether resizing this edge changes the window's height.
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(
            self,
            Self::Top
                | Self::Bottom
                | Self::TopLeft
                | Self::TopRight
                | Self::BottomLeft
                | Self::BottomRight
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ResizeEdge;

    #[test]
    fn is_corner_only_for_corners() {
        assert!(ResizeEdge::TopLeft.is_corner());
        assert!(ResizeEdge::BottomRight.is_corner());
        assert!(!ResizeEdge::Top.is_corner());
        assert!(!ResizeEdge::Left.is_corner());
    }

    #[test]
    fn axis_predicates() {
        assert!(ResizeEdge::Left.is_horizontal());
        assert!(!ResizeEdge::Left.is_vertical());
        assert!(ResizeEdge::Top.is_vertical());
        assert!(!ResizeEdge::Top.is_horizontal());
        assert!(ResizeEdge::TopLeft.is_horizontal());
        assert!(ResizeEdge::TopLeft.is_vertical());
    }
}
