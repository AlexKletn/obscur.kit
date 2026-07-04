//! `DocumentSymbol`: a hierarchical outline entry for one `.okui` element or
//! component, mirroring the LSP `DocumentSymbol` shape.

use crate::diagnostics::Range;

/// Whether a [`DocumentSymbol`] names a plain element or a component.
///
/// Following the okui lowering convention, a tag whose name starts with an
/// uppercase letter is a component; everything else is an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    /// A lowercase-tagged element (e.g. `<row>`).
    Element,
    /// An uppercase-tagged component (e.g. `<Button>`).
    Component,
}

impl SymbolKind {
    /// Classifies a tag `name` by the case of its first character.
    #[must_use]
    pub fn of_tag(name: &str) -> Self {
        if name.chars().next().is_some_and(char::is_uppercase) {
            Self::Component
        } else {
            Self::Element
        }
    }

    /// Returns a short lowercase label for CLI output.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Element => "element",
            Self::Component => "component",
        }
    }
}

/// A hierarchical outline entry for one element/component in a `.okui` document.
///
/// Produced by [`Analyzer::document_symbols`](crate::Analyzer::document_symbols),
/// it carries the tag `name`, its [`SymbolKind`], the source `range` it covers,
/// and its nested `children`, so a client can render a `textDocument/documentSymbol`
/// outline.
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
