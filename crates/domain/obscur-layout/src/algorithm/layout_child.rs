#![doc = include_str!("layout_child.doc/_module.md")]

use obscur_identity::NodeId;

use crate::model::LayoutStyle;

#[doc = include_str!("layout_child.doc/LayoutChild.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutChild {
    #[doc = include_str!("layout_child.doc/node.md")]
    pub node: NodeId,
    #[doc = include_str!("layout_child.doc/style.md")]
    pub style: LayoutStyle,
}

impl LayoutChild {
    #[doc = include_str!("layout_child.doc/new.md")]
    #[must_use]
    pub const fn new(node: NodeId, style: LayoutStyle) -> Self {
        Self { node, style }
    }

    #[doc = include_str!("layout_child.doc/node_2.md")]
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }

    #[doc = include_str!("layout_child.doc/style_2.md")]
    #[must_use]
    pub const fn style(self) -> LayoutStyle {
        self.style
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutChild;
    use crate::model::{Dimension, LayoutStyle};
    use obscur_identity::NodeId;

    #[test]
    fn new_stores_node_and_style() {
        let style = LayoutStyle::default().with_width(Dimension::Px(24.0));
        let child = LayoutChild::new(NodeId::from_raw(7), style);
        assert_eq!(child.node(), NodeId::from_raw(7));
        assert_eq!(child.style(), style);
    }

    #[test]
    fn is_copy_and_eq() {
        let child = LayoutChild::new(NodeId::from_raw(1), LayoutStyle::default());
        let copy = child;
        assert_eq!(child, copy);
    }
}
