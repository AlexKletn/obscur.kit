//! `LayoutChild`: a child's identity and style, as seen by a layout algorithm.

use obscur_identity::NodeId;

use crate::model::LayoutStyle;

/// One child of a node, presented to a [`LayoutAlgorithm`] as identity + style.
///
/// [`LayoutAlgorithm`]: super::LayoutAlgorithm
///
/// Layout algorithms in this crate never walk the retained tree themselves — the
/// runtime does (spec §4.2). An algorithm receives the parent's style, the slice
/// of its children (each a `LayoutChild`), and a [`ChildMeasurer`] callback used
/// to size a child under chosen constraints. This type carries exactly what an
/// algorithm needs about a child *before* measuring it: which node it is and the
/// style that drives its participation (flex basis, grow/shrink, explicit sizes,
/// alignment overrides, ...).
///
/// [`ChildMeasurer`]: super::ChildMeasurer
///
/// All fields are `Copy`; the whole value is cheap to pass by value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutChild {
    /// Stable identity of the child node.
    pub node: NodeId,
    /// The child's own layout input style.
    pub style: LayoutStyle,
}

impl LayoutChild {
    /// Creates a child descriptor from its node identity and layout style.
    #[must_use]
    pub const fn new(node: NodeId, style: LayoutStyle) -> Self {
        Self { node, style }
    }

    /// The child's node identity.
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }

    /// The child's layout input style.
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
