//! `Image`: a raster/vector image primitive (spec §2.6, §5.16).

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_view::view::{View, ViewNode, ViewProperty, ViewValue};

/// A reference to an image resource drawn into the view tree.
///
/// `Image` is one of the minimal built-in primitives (spec §2.6). It names an
/// image by an opaque `source` key that the resource hub resolves to real pixel
/// or vector data (spec §5.16), and optionally an `alt` accessible name. It is
/// a leaf element primitive: it lowers to a [`ViewNode::Element`] carrying its
/// source (and alt, when present) as properties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    source: Box<str>,
    alt: Option<Box<str>>,
}

impl Image {
    /// Stable component id for the image primitive (see [`Container::COMPONENT_ID`]
    /// for the shared id scheme).
    ///
    /// [`Container::COMPONENT_ID`]: crate::Container::COMPONENT_ID
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00C0_0003);

    /// Property name for the image source key.
    ///
    /// Prop-name symbols share the `0x00C1_NNXX` space, where `NN` mirrors the
    /// owning primitive's component-id nibble and `XX` counts its properties;
    /// these are the stable raw symbol ids the style/reconcile layer maps.
    pub const PROP_SRC: InternedSymbol = InternedSymbol::from_raw(0x00C1_0301);
    /// Property name for the accessible alternative text.
    pub const PROP_ALT: InternedSymbol = InternedSymbol::from_raw(0x00C1_0302);

    /// Creates an image referencing `source` with no alternative text.
    #[must_use]
    pub fn new(source: impl Into<Box<str>>) -> Self {
        Self {
            source: source.into(),
            alt: None,
        }
    }

    /// Sets the accessible alternative text, returning the updated image.
    #[must_use]
    pub fn with_alt(mut self, alt: impl Into<Box<str>>) -> Self {
        self.alt = Some(alt.into());
        self
    }

    /// Lowers this image into its [`ViewNode::Element`], emitting the source
    /// property and, when set, the alt property.
    #[must_use]
    pub fn into_view_node(self) -> ViewNode {
        let mut node = ViewNode::element(Self::COMPONENT_ID).with_prop(ViewProperty::new(
            Self::PROP_SRC,
            ViewValue::Text(self.source),
        ));
        if let Some(alt) = self.alt {
            node = node.with_prop(ViewProperty::new(Self::PROP_ALT, ViewValue::Text(alt)));
        }
        node
    }

    /// Lowers this image into a single-node [`View`].
    #[must_use]
    pub fn view(self) -> View {
        View::single(self.into_view_node())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_source_only_without_alt() {
        let node = Image::new("logo.png").into_view_node();
        if let ViewNode::Element {
            component, props, ..
        } = node
        {
            assert_eq!(component, Image::COMPONENT_ID);
            assert_eq!(props.len(), 1);
            assert_eq!(props[0].name(), Image::PROP_SRC);
            assert_eq!(props[0].value(), &ViewValue::Text("logo.png".into()));
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn emits_alt_when_set() {
        let node = Image::new("logo.png")
            .with_alt("Company logo")
            .into_view_node();
        if let ViewNode::Element { props, .. } = node {
            assert_eq!(props.len(), 2);
            assert_eq!(props[1].name(), Image::PROP_ALT);
            assert_eq!(props[1].value(), &ViewValue::Text("Company logo".into()));
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn view_wraps_single_node() {
        let view = Image::new("x").view();
        assert_eq!(view.root().len(), 1);
    }
}
