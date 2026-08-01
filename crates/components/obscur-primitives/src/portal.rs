#![doc = include_str!("portal.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("portal.doc/Portal.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Portal {
    target: Box<str>,
    children: Vec<ViewNode>,
}

impl Portal {
    #[doc = include_str!("portal.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00C0_0006);

    #[doc = include_str!("portal.doc/PROP_TARGET.md")]
    pub const PROP_TARGET: InternedSymbol = InternedSymbol::from_raw(0x00C1_0601);

    #[doc = include_str!("portal.doc/new.md")]
    #[must_use]
    pub fn new(target: impl Into<Box<str>>) -> Self {
        Self {
            target: target.into(),
            children: Vec::new(),
        }
    }

    #[doc = include_str!("portal.doc/child.md")]
    #[must_use]
    pub fn child(mut self, node: ViewNode) -> Self {
        self.children.push(node);
        self
    }

    #[doc = include_str!("portal.doc/into_view_node.md")]
    #[must_use]
    pub fn into_view_node(self) -> ViewNode {
        ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(
                Self::PROP_TARGET,
                ViewValue::Text(self.target),
            ))
            .with_children(ViewFragment::from_nodes(self.children))
    }

    #[doc = include_str!("portal.doc/view.md")]
    #[must_use]
    pub fn view(self) -> View {
        View::single(self.into_view_node())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_target_and_children() {
        let node = Portal::new("overlay-root")
            .child(ViewNode::text("dialog"))
            .into_view_node();
        if let ViewNode::Element {
            component,
            props,
            children,
            ..
        } = node
        {
            assert_eq!(component, Portal::COMPONENT_ID);
            assert_eq!(props[0].name(), Portal::PROP_TARGET);
            assert_eq!(props[0].value(), &ViewValue::Text("overlay-root".into()));
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn view_wraps_single_node() {
        let view = Portal::new("t").view();
        assert_eq!(view.root().len(), 1);
    }
}
