#![doc = include_str!("container.doc/_module.md")]

use obscur_identity::ComponentId;
use obscur_view::view::{View, ViewFragment, ViewNode};

#[doc = include_str!("container.doc/Container.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Container {
    children: Vec<ViewNode>,
}

impl Container {
    #[doc = include_str!("container.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00C0_0001);

    #[doc = include_str!("container.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    #[doc = include_str!("container.doc/child.md")]
    #[must_use]
    pub fn child(mut self, node: ViewNode) -> Self {
        self.children.push(node);
        self
    }

    #[doc = include_str!("container.doc/with_children.md")]
    #[must_use]
    pub fn with_children(mut self, nodes: impl IntoIterator<Item = ViewNode>) -> Self {
        self.children.extend(nodes);
        self
    }

    #[doc = include_str!("container.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.children.len()
    }

    #[doc = include_str!("container.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    #[doc = include_str!("container.doc/into_view_node.md")]
    #[must_use]
    pub fn into_view_node(self) -> ViewNode {
        ViewNode::element(Self::COMPONENT_ID).with_children(ViewFragment::from_nodes(self.children))
    }

    #[doc = include_str!("container.doc/view.md")]
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
