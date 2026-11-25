//! `DocumentSymbol`: a hierarchical outline entry for one `.okui` element or
//! component, mirroring the LSP `DocumentSymbol` shape.

use crate::diagnostics::Range;

use super::SymbolKind;

#[doc = include_str!("../../docs/api/analysis/document_symbol.doc/DocumentSymbol.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSymbol {
    /// The element/component tag name.
    pub name: Box<str>,
    /// Whether the symbol is an element or a component.
    pub kind: SymbolKind,
    /// The full source span the element covers.
    pub range: Range,
    /// The nested child symbols (elements/components inside this one).
    pub children: Vec<DocumentSymbol>,
}

impl DocumentSymbol {
    /// Creates a leaf symbol (no children) for `name` of `kind` over `range`.
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, kind: SymbolKind, range: Range) -> Self {
        Self {
            name: name.into(),
            kind,
            range,
            children: Vec::new(),
        }
    }

    /// Builder: sets the symbol's children.
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
