#![doc = include_str!("image.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_view::view::{SRC, View, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("image.doc/Image.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    source: Box<str>,
    alt: Option<Box<str>>,
}

impl Image {
    #[doc = include_str!("image.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00C0_0003);

    #[doc = include_str!("image.doc/PROP_SRC.md")]
    pub const PROP_SRC: InternedSymbol = SRC;
    #[doc = include_str!("image.doc/PROP_ALT.md")]
    pub const PROP_ALT: InternedSymbol = InternedSymbol::from_raw(0x00C1_0302);

    #[doc = include_str!("image.doc/new.md")]
    #[must_use]
    pub fn new(source: impl Into<Box<str>>) -> Self {
        Self {
            source: source.into(),
            alt: None,
        }
    }

    #[doc = include_str!("image.doc/with_alt.md")]
    #[must_use]
    pub fn with_alt(mut self, alt: impl Into<Box<str>>) -> Self {
        self.alt = Some(alt.into());
        self
    }

    #[doc = include_str!("image.doc/into_view_node.md")]
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

    #[doc = include_str!("image.doc/view.md")]
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
