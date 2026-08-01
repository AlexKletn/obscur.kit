#![doc = include_str!("scroll.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("scroll.doc/Scroll.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Scroll {
    children: Vec<ViewNode>,
    horizontal: bool,
    vertical: bool,
}

impl Default for Scroll {
    #[doc = include_str!("scroll.doc/default.md")]
    fn default() -> Self {
        Self {
            children: Vec::new(),
            horizontal: false,
            vertical: true,
        }
    }
}

impl Scroll {
    #[doc = include_str!("scroll.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00C0_0004);

    #[doc = include_str!("scroll.doc/PROP_SCROLL_X.md")]
    pub const PROP_SCROLL_X: InternedSymbol = InternedSymbol::from_raw(0x00C1_0401);
    #[doc = include_str!("scroll.doc/PROP_SCROLL_Y.md")]
    pub const PROP_SCROLL_Y: InternedSymbol = InternedSymbol::from_raw(0x00C1_0402);

    #[doc = include_str!("scroll.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("scroll.doc/child.md")]
    #[must_use]
    pub fn child(mut self, node: ViewNode) -> Self {
        self.children.push(node);
        self
    }

    #[doc = include_str!("scroll.doc/horizontal.md")]
    #[must_use]
    pub const fn horizontal(mut self, enabled: bool) -> Self {
        self.horizontal = enabled;
        self
    }

    #[doc = include_str!("scroll.doc/vertical.md")]
    #[must_use]
    pub const fn vertical(mut self, enabled: bool) -> Self {
        self.vertical = enabled;
        self
    }

    #[doc = include_str!("scroll.doc/into_view_node.md")]
    #[must_use]
    pub fn into_view_node(self) -> ViewNode {
        ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(
                Self::PROP_SCROLL_X,
                ViewValue::Bool(self.horizontal),
            ))
            .with_prop(ViewProperty::new(
                Self::PROP_SCROLL_Y,
                ViewValue::Bool(self.vertical),
            ))
            .with_children(ViewFragment::from_nodes(self.children))
    }

    #[doc = include_str!("scroll.doc/view.md")]
    #[must_use]
    pub fn view(self) -> View {
        View::single(self.into_view_node())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_vertical_only() {
        let s = Scroll::new();
        assert!(!s.horizontal);
        assert!(s.vertical);
    }

    #[test]
    fn emits_axis_flags_and_children() {
        let node = Scroll::new()
            .horizontal(true)
            .vertical(false)
            .child(ViewNode::text("a"))
            .into_view_node();
        if let ViewNode::Element {
            component,
            props,
            children,
            ..
        } = node
        {
            assert_eq!(component, Scroll::COMPONENT_ID);
            assert_eq!(props[0].name(), Scroll::PROP_SCROLL_X);
            assert_eq!(props[0].value(), &ViewValue::Bool(true));
            assert_eq!(props[1].name(), Scroll::PROP_SCROLL_Y);
            assert_eq!(props[1].value(), &ViewValue::Bool(false));
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn view_wraps_single_node() {
        let view = Scroll::new().view();
        assert_eq!(view.root().len(), 1);
    }
}
