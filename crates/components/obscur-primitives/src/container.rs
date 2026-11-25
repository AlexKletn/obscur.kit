//! `Container`: a grouping box primitive (spec §2.6).

use obscur_identity::ComponentId;
use obscur_view::view::{View, ViewFragment, ViewNode};

#[doc = include_str!("../docs/api/container.doc/Container.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Container {
    children: Vec<ViewNode>,
}

impl Container {
    #[doc = include_str!("../docs/api/container.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00C0_0001);

    /// Creates an empty container with no children.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Appends a child node, returning the updated container.
    #[must_use]
    pub fn child(mut self, node: ViewNode) -> Self {
        self.children.push(node);
        self
    }

    /// Appends every node from `nodes`, returning the updated container.
    #[must_use]
    pub fn with_children(mut self, nodes: impl IntoIterator<Item = ViewNode>) -> Self {
        self.children.extend(nodes);
        self
    }

    /// Returns the number of direct children.
    #[must_use]
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns `true` if the container has no children.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Lowers this container into its [`ViewNode::Element`] with no props and
    /// its children as the element's fragment.
    #[must_use]
    pub fn into_view_node(self) -> ViewNode {
        ViewNode::element(Self::COMPONENT_ID).with_children(ViewFragment::from_nodes(self.children))
    }

    /// Lowers this container into a single-node [`View`].
    #[must_use]
    pub fn view(self) -> View {
        View::single(self.into_view_node())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_by_default() {
        let c = Container::new();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn collects_children_in_order() {
        let c = Container::new()
            .child(ViewNode::text("a"))
            .with_children([ViewNode::text("b"), ViewNode::text("c")]);
        assert_eq!(c.len(), 3);

        let node = c.into_view_node();
        if let ViewNode::Element {
            component,
            props,
            children,
            ..
        } = node
        {
            assert_eq!(component, Container::COMPONENT_ID);
            assert!(props.is_empty());
            assert_eq!(children.len(), 3);
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn view_wraps_single_node() {
        let view = Container::new().view();
        assert_eq!(view.root().len(), 1);
    }
}
