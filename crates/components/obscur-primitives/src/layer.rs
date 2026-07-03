//! `Layer`: a compositing-layer primitive (spec §2.6, §5.1).

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A compositing layer that groups children with an applied opacity.
///
/// `Layer` is one of the minimal built-in primitives (spec §2.6). It maps to a
/// scene compositing layer (spec §5.1): its children are rendered into an
/// isolated layer that is then composited with the layer's `opacity`, so
/// group-level transparency and blending apply as one unit. It is a leaf
/// element primitive: it lowers to a [`ViewNode::Element`] carrying its opacity
/// as a property and its children as the element's fragment.
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    children: Vec<ViewNode>,
    opacity: f32,
}

impl Default for Layer {
    /// A fully opaque layer with no children.
    fn default() -> Self {
        Self {
            children: Vec::new(),
            opacity: 1.0,
        }
    }
}

impl Layer {
    /// Stable component id for the layer primitive (see
    /// [`Container::COMPONENT_ID`] for the shared id scheme).
    ///
    /// [`Container::COMPONENT_ID`]: crate::Container::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00C0_0007);

    /// Property name for the layer opacity (see [`Image::PROP_SRC`] for the
    /// shared prop-symbol scheme).
    ///
    /// [`Image::PROP_SRC`]: crate::Image::PROP_SRC
    pub const PROP_OPACITY: InternedSymbol = InternedSymbol::from_raw(0x00C1_0701);

    /// Creates a default, fully opaque layer with no children.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a child node, returning the updated layer.
    #[must_use]
    pub fn child(mut self, node: ViewNode) -> Self {
        self.children.push(node);
        self
    }

    /// Sets the layer opacity, clamped to `0.0..=1.0`, returning the updated
    /// layer.
    #[must_use]
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Lowers this layer into its [`ViewNode::Element`], emitting the opacity
    /// property and its children as the fragment.
    #[must_use]
    pub fn into_view_node(self) -> ViewNode {
        ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(
                Self::PROP_OPACITY,
                ViewValue::Float(f64::from(self.opacity)),
            ))
            .with_children(ViewFragment::from_nodes(self.children))
    }

    /// Lowers this layer into a single-node [`View`].
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
