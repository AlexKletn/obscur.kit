#![doc = include_str!("layer_edge.doc/_module.md")]

#[doc = include_str!("layer_edge.doc/LayerEdge.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerEdge {
    #[doc = include_str!("layer_edge.doc/Top.md")]
    Top,
    #[doc = include_str!("layer_edge.doc/Right.md")]
    Right,
    #[doc = include_str!("layer_edge.doc/Bottom.md")]
    Bottom,
    #[doc = include_str!("layer_edge.doc/Left.md")]
    Left,
}

impl LayerEdge {
    #[doc = include_str!("layer_edge.doc/opposite.md")]
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
        }
    }

    #[doc = include_str!("layer_edge.doc/is_horizontal.md")]
    #[must_use]
    pub const fn is_horizontal(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVERY_EDGE: [LayerEdge; 4] = [
        LayerEdge::Top,
        LayerEdge::Right,
        LayerEdge::Bottom,
        LayerEdge::Left,
    ];

    #[test]
    fn opposing_an_edge_twice_returns_it() {
        for edge in EVERY_EDGE {
            assert_eq!(edge.opposite().opposite(), edge);
        }
    }

    #[test]
    fn an_edge_and_its_opposite_share_an_axis() {
        for edge in EVERY_EDGE {
            assert_eq!(edge.is_horizontal(), edge.opposite().is_horizontal());
        }
    }

    #[test]
    fn a_bar_along_the_top_runs_horizontally() {
        assert!(LayerEdge::Top.is_horizontal());
        assert!(LayerEdge::Bottom.is_horizontal());
        assert!(!LayerEdge::Left.is_horizontal());
        assert!(!LayerEdge::Right.is_horizontal());
    }
}
