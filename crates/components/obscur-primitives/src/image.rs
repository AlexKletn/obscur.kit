#![doc = include_str!("image.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_view::view::{LOCAL_NAME, SRC, View, ViewNode, ViewProperty, ViewValue};

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
        // The element names itself `img`, under the reserved LOCAL_NAME symbol the
        // matcher reads as a node's CSS *type* (spec §4.13) — the same name an
        // authored `<img>` tag contributes, and the same name this primitive is
        // already the built-in for (`name_binding::builtin_component_id("img")`).
        // Without it the two spellings were one component id over two different
        // trees: the UA sheet's `img { … }` rule matched the markup and not the
        // primitive, and the accessibility projection announced an unnamed group
        // rather than `SemanticRole::Image`, which it derives from a local name.
        let mut node = ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::from("img")))
            .with_prop(ViewProperty::new(
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

    fn prop(props: &[ViewProperty], name: InternedSymbol) -> Option<&ViewValue> {
        props
            .iter()
            .find(|p| p.name() == name)
            .map(ViewProperty::value)
    }

    #[test]
    fn emits_source_only_without_alt() {
        let node = Image::new("logo.png").into_view_node();
        if let ViewNode::Element {
            component, props, ..
        } = node
        {
            assert_eq!(component, Image::COMPONENT_ID);
            // Name + source, and no alt. These used to be asserted by position
            // (`props.len() == 1`, `props[0]`) against an element that carried
            // nothing but its source — the shape that made `img { … }` unable to
            // reach it. The claims about src and alt are unchanged; only the way
            // they are read is, so that adding a property is not the same event as
            // changing one.
            assert_eq!(props.len(), 2);
            assert_eq!(
                prop(&props, Image::PROP_SRC),
                Some(&ViewValue::Text("logo.png".into()))
            );
            assert_eq!(prop(&props, Image::PROP_ALT), None);
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
            assert_eq!(props.len(), 3);
            assert_eq!(
                prop(&props, Image::PROP_ALT),
                Some(&ViewValue::Text("Company logo".into()))
            );
        } else {
            panic!("expected element");
        }
    }

    #[test]
    fn the_primitive_names_itself_img() {
        // What a `img { … }` type selector matches, and what the accessibility
        // projection derives `SemanticRole::Image` from. `<img>` in markup and
        // `Image::new(…)` in Rust are one built-in — `builtin_component_id("img")`
        // is this very `COMPONENT_ID` — and until this name existed they were that
        // one id over two different trees: the tag contributed a local name and
        // the primitive contributed none, so one UA rule reached only one of them.
        let node = Image::new("logo.png").into_view_node();
        let ViewNode::Element { props, .. } = node else {
            panic!("expected element");
        };
        assert_eq!(
            prop(&props, LOCAL_NAME),
            Some(&ViewValue::Text("img".into()))
        );
    }

    #[test]
    fn view_wraps_single_node() {
        let view = Image::new("x").view();
        assert_eq!(view.root().len(), 1);
    }
}
