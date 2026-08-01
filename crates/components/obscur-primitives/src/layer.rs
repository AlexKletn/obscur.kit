#![doc = include_str!("layer.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("layer.doc/Layer.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    children: Vec<ViewNode>,
    opacity: f32,
}

impl Default for Layer {
    #[doc = include_str!("layer.doc/default.md")]
    fn default() -> Self {
        Self {
            children: Vec::new(),
            opacity: 1.0,
        }
    }
}

impl Layer {
    #[doc = include_str!("layer.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00C0_0007);

    #[doc = include_str!("layer.doc/PROP_OPACITY.md")]
    pub const PROP_OPACITY: InternedSymbol = InternedSymbol::from_raw(0x00C1_0701);

    #[doc = include_str!("layer.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("layer.doc/child.md")]
    #[must_use]
    pub fn child(mut self, node: ViewNode) -> Self {
        self.children.push(node);
        self
    }

    #[doc = include_str!("layer.doc/with_opacity.md")]
    #[must_use]
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    #[doc = include_str!("layer.doc/into_view_node.md")]
    #[must_use]
    pub fn into_view_node(self) -> ViewNode {
        ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(
                Self::PROP_OPACITY,
                ViewValue::Float(f64::from(self.opacity)),
            ))
            .with_children(ViewFragment::from_nodes(self.children))
    }

    #[doc = include_str!("layer.doc/view.md")]
    #[must_use]
    pub fn view(self) -> View {
        View::single(self.into_view_node())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_opaque() {
        assert_eq!(Layer::new().opacity, 1.0);
    }

    #[test]
    fn clamps_opacity() {
        assert_eq!(Layer::new().with_opacity(2.0).opacity, 1.0);
        assert_eq!(Layer::new().with_opacity(-1.0).opacity, 0.0);
    }

    #[test]
    fn emits_opacity_and_children() {
        let node = Layer::new()
            .with_opacity(0.5)
            .child(ViewNode::text("a"))
            .into_view_node();
        if let ViewNode::Element {
            component,
            props,
            children,
            ..
        } = node
        {
            assert_eq!(component, Layer::COMPONENT_ID);
            assert_eq!(props[0].name(), Layer::PROP_OPACITY);
            assert_eq!(props[0].value(), &ViewValue::Float(0.5));
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn view_wraps_single_node() {
        let view = Layer::new().view();
        assert_eq!(view.root().len(), 1);
    }
}
