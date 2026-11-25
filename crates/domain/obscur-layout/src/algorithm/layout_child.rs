//! `LayoutChild`: a child's identity and style, as seen by a layout algorithm.

use obscur_identity::NodeId;

use crate::model::LayoutStyle;

#[doc = include_str!("../../docs/api/algorithm/layout_child.doc/LayoutChild.md")]
#[derive(Debug, Clone, PartialEq)]
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
    pub const fn node(&self) -> NodeId {
        self.node
    }

    /// The child's layout input style.
    #[must_use]
    pub const fn style(&self) -> &LayoutStyle {
        &self.style
    }
}

/// Provides a permutation when the children are not in their layout order.
pub(super) fn ordered_child_indices(children: &[LayoutChild]) -> Option<Vec<usize>> {
    if children
        .windows(2)
        .all(|pair| pair[0].style.order <= pair[1].style.order)
    {
        return None;
    }
    let mut indices: Vec<_> = (0..children.len()).collect();
    indices.sort_unstable_by_key(|&i| (children[i].style.order, i));
    Some(indices)
}

#[cfg(test)]
mod tests {
    use super::LayoutChild;
    use crate::model::{Dimension, LayoutStyle};
    use obscur_identity::NodeId;

    #[test]
    fn new_stores_node_and_style() {
        let style = LayoutStyle::default().with_width(Dimension::Px(24.0));
        let child = LayoutChild::new(NodeId::from_raw(7), style.clone());
        assert_eq!(child.node(), NodeId::from_raw(7));
        assert_eq!(child.style(), &style);
    }

    #[test]
    fn is_clone_and_eq() {
        let child = LayoutChild::new(NodeId::from_raw(1), LayoutStyle::default());
        let copy = child.clone();
        assert_eq!(child, copy);
    }
}
