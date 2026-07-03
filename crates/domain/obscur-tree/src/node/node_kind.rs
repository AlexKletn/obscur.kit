//! Classification of a retained node.

use obscur_identity::{ComponentId, InternedSymbol};

/// The classification of a retained tree node.
///
/// Every retained node carries exactly one `NodeKind` describing what it
/// represents in the tree. Text is a real node (spec §4.16), and
/// system-generated parts are modelled as [`NodeKind::Synthetic`] retained
/// nodes — hidden from the component tree but visible in DevTools.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// The tree root / a structural grouping node with no component or content
    /// of its own.
    Root,
    /// An instantiated component element.
    Element {
        /// Identity of the component instance backing this element.
        component: ComponentId,
    },
    /// A text node carrying literal content (spec §4.16).
    Text {
        /// The text content of the node.
        content: Box<str>,
    },
    /// A system-generated ("synthetic") part, hidden from the component tree but
    /// visible in DevTools (spec §4.16). The tag names which part this is.
    Synthetic {
        /// Interned symbol naming the synthetic part.
        tag: InternedSymbol,
    },
}

impl NodeKind {
    /// Returns `true` if this is the [`NodeKind::Root`] grouping node.
    #[must_use]
    pub const fn is_root(&self) -> bool {
        matches!(self, Self::Root)
    }

    /// Returns `true` if this is a [`NodeKind::Element`].
    #[must_use]
    pub const fn is_element(&self) -> bool {
        matches!(self, Self::Element { .. })
    }

    /// Returns `true` if this is a [`NodeKind::Text`] node.
    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    /// Returns `true` if this is a system-generated [`NodeKind::Synthetic`]
    /// node.
    #[must_use]
    pub const fn is_synthetic(&self) -> bool {
        matches!(self, Self::Synthetic { .. })
    }

    /// Returns the backing [`ComponentId`] if this is an element node.
    #[must_use]
    pub const fn component_id(&self) -> Option<ComponentId> {
        match self {
            Self::Element { component } => Some(*component),
            _ => None,
        }
    }

    /// Returns the text content if this is a text node.
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Text { content } => Some(content),
            _ => None,
        }
    }

    /// Returns the synthetic part tag if this is a synthetic node.
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
