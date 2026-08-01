#![doc = include_str!("normalized_node.doc/_module.md")]

use obscur_geometry::Rect;
use obscur_identity::NodeId;

#[doc = include_str!("normalized_node.doc/NormalizedNode.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedNode {
    #[doc = include_str!("normalized_node.doc/node.md")]
    node: NodeId,
    #[doc = include_str!("normalized_node.doc/opacity.md")]
    opacity: Option<f32>,
    #[doc = include_str!("normalized_node.doc/layout_bounds.md")]
    layout_bounds: Option<Rect>,
}

impl NormalizedNode {
    #[doc = include_str!("normalized_node.doc/new.md")]
    #[must_use]
    pub fn new(node: NodeId, opacity: Option<f32>, layout_bounds: Option<Rect>) -> Self {
        Self {
            node,
            opacity,
            layout_bounds,
        }
    }

    #[doc = include_str!("normalized_node.doc/node_2.md")]
    #[must_use]
    pub fn node(&self) -> NodeId {
        self.node
    }

    #[doc = include_str!("normalized_node.doc/opacity_2.md")]
    #[must_use]
    pub fn opacity(&self) -> Option<f32> {
        self.opacity
    }

    #[doc = include_str!("normalized_node.doc/layout_bounds_2.md")]
    #[must_use]
    pub fn layout_bounds(&self) -> Option<Rect> {
        self.layout_bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Size};

    #[test]
    fn accessors_read_fields() {
        let bounds = Rect::new(Point::new(1.0, 2.0), Size::new(3.0, 4.0));
        let row = NormalizedNode::new(NodeId::from_raw(7), Some(0.5), Some(bounds));
        assert_eq!(row.node(), NodeId::from_raw(7));
        assert_eq!(row.opacity(), Some(0.5));
        assert_eq!(row.layout_bounds(), Some(bounds));
    }

    #[test]
    fn absent_stage_data_is_none() {
        let row = NormalizedNode::new(NodeId::from_raw(1), None, None);
        assert_eq!(row.opacity(), None);
        assert_eq!(row.layout_bounds(), None);
    }

    #[test]
    fn equal_rows_compare_equal() {
        let a = NormalizedNode::new(NodeId::from_raw(3), Some(1.0), None);
        let b = NormalizedNode::new(NodeId::from_raw(3), Some(1.0), None);
        assert_eq!(a, b);
    }
}
