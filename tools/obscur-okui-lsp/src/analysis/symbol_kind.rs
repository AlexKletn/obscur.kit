#[doc = include_str!("../../docs/api/analysis/document_symbol.doc/SymbolKind.md")]
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
