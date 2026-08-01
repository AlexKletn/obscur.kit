#![doc = include_str!("document_symbol.doc/_module.md")]

use crate::diagnostics::Range;

#[doc = include_str!("document_symbol.doc/SymbolKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    #[doc = include_str!("document_symbol.doc/Element.md")]
    Element,
    #[doc = include_str!("document_symbol.doc/Component.md")]
    Component,
}

impl SymbolKind {
    #[doc = include_str!("document_symbol.doc/of_tag.md")]
    #[must_use]
    pub fn of_tag(name: &str) -> Self {
        if name.chars().next().is_some_and(char::is_uppercase) {
            Self::Component
        } else {
            Self::Element
        }
    }

    #[doc = include_str!("document_symbol.doc/label.md")]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Element => "element",
            Self::Component => "component",
        }
    }
}

#[doc = include_str!("document_symbol.doc/DocumentSymbol.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSymbol {
    #[doc = include_str!("document_symbol.doc/name.md")]
    pub name: Box<str>,
    #[doc = include_str!("document_symbol.doc/kind.md")]
    pub kind: SymbolKind,
    #[doc = include_str!("document_symbol.doc/range.md")]
    pub range: Range,
    #[doc = include_str!("document_symbol.doc/children.md")]
    pub children: Vec<DocumentSymbol>,
}

impl DocumentSymbol {
    #[doc = include_str!("document_symbol.doc/new.md")]
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, kind: SymbolKind, range: Range) -> Self {
        Self {
            name: name.into(),
            kind,
            range,
            children: Vec::new(),
        }
    }

    #[doc = include_str!("document_symbol.doc/with_children.md")]
    #[must_use]
    pub fn with_children(mut self, children: Vec<DocumentSymbol>) -> Self {
        self.children = children;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_tag_by_case() {
        assert_eq!(SymbolKind::of_tag("Button"), SymbolKind::Component);
        assert_eq!(SymbolKind::of_tag("row"), SymbolKind::Element);
        assert_eq!(SymbolKind::of_tag(""), SymbolKind::Element);
    }

    #[test]
    fn stores_children() {
        use crate::diagnostics::Position;
        let range = Range::new(Position::new(0, 0), Position::new(0, 4));
        let child = DocumentSymbol::new("Icon", SymbolKind::Component, range);
        let parent =
            DocumentSymbol::new("Row", SymbolKind::Component, range).with_children(vec![child]);
        assert_eq!(parent.children.len(), 1);
        assert_eq!(&*parent.children[0].name, "Icon");
    }
}
