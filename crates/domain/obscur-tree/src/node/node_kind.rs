#![doc = include_str!("node_kind.doc/_module.md")]

use obscur_identity::{ComponentId, InternedSymbol};

#[doc = include_str!("node_kind.doc/NodeKind.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    #[doc = include_str!("node_kind.doc/Root.md")]
    Root,
    #[doc = include_str!("node_kind.doc/Element.md")]
    Element {
        #[doc = include_str!("node_kind.doc/component.md")]
        component: ComponentId,
    },
    #[doc = include_str!("node_kind.doc/Text.md")]
    Text {
        #[doc = include_str!("node_kind.doc/content.md")]
        content: Box<str>,
    },
    #[doc = include_str!("node_kind.doc/Synthetic.md")]
    Synthetic {
        #[doc = include_str!("node_kind.doc/tag.md")]
        tag: InternedSymbol,
    },
}

impl NodeKind {
    #[doc = include_str!("node_kind.doc/is_root.md")]
    #[must_use]
    pub const fn is_root(&self) -> bool {
        matches!(self, Self::Root)
    }

    #[doc = include_str!("node_kind.doc/is_element.md")]
    #[must_use]
    pub const fn is_element(&self) -> bool {
        matches!(self, Self::Element { .. })
    }

    #[doc = include_str!("node_kind.doc/is_text.md")]
    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    #[doc = include_str!("node_kind.doc/is_synthetic.md")]
    #[must_use]
    pub const fn is_synthetic(&self) -> bool {
        matches!(self, Self::Synthetic { .. })
    }

    #[doc = include_str!("node_kind.doc/component_id.md")]
    #[must_use]
    pub const fn component_id(&self) -> Option<ComponentId> {
        match self {
            Self::Element { component } => Some(*component),
            _ => None,
        }
    }

    #[doc = include_str!("node_kind.doc/text.md")]
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Text { content } => Some(content),
            _ => None,
        }
    }

    #[doc = include_str!("node_kind.doc/synthetic_tag.md")]
    #[must_use]
    pub const fn synthetic_tag(&self) -> Option<InternedSymbol> {
        match self {
            Self::Synthetic { tag } => Some(*tag),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_predicates() {
        let k = NodeKind::Root;
        assert!(k.is_root());
        assert!(!k.is_element());
        assert_eq!(k.component_id(), None);
    }

    #[test]
    fn element_exposes_component() {
        let c = ComponentId::from_raw(7);
        let k = NodeKind::Element { component: c };
        assert!(k.is_element());
        assert_eq!(k.component_id(), Some(c));
        assert_eq!(k.text(), None);
    }

    #[test]
    fn text_exposes_content() {
        let k = NodeKind::Text {
            content: "hello".into(),
        };
        assert!(k.is_text());
        assert_eq!(k.text(), Some("hello"));
        assert_eq!(k.component_id(), None);
    }

    #[test]
    fn synthetic_exposes_tag() {
        let tag = InternedSymbol::from_raw(3);
        let k = NodeKind::Synthetic { tag };
        assert!(k.is_synthetic());
        assert_eq!(k.synthetic_tag(), Some(tag));
    }
}
