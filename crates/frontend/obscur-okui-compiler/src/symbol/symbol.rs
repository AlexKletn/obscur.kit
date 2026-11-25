//! `Symbol`: a named entity visible within a lexical scope (spec §2.10).

use super::SymbolKind;

#[doc = include_str!("../../docs/api/symbol/symbol.doc/Symbol.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    name: Box<str>,
    kind: SymbolKind,
}

impl Symbol {
    /// Creates a symbol with the given `name` and `kind`.
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, kind: SymbolKind) -> Self {
        Self {
            name: name.into(),
            kind,
        }
    }

    /// Creates a [`Prop`](SymbolKind::Prop) symbol.
    #[must_use]
    pub fn prop(name: impl Into<Box<str>>) -> Self {
        Self::new(name, SymbolKind::Prop)
    }

    /// Creates a [`State`](SymbolKind::State) symbol.
    #[must_use]
    pub fn state(name: impl Into<Box<str>>) -> Self {
        Self::new(name, SymbolKind::State)
    }

    /// Creates a [`LoopLocal`](SymbolKind::LoopLocal) symbol.
    #[must_use]
    pub fn loop_local(name: impl Into<Box<str>>) -> Self {
        Self::new(name, SymbolKind::LoopLocal)
    }

    /// Creates a [`Component`](SymbolKind::Component) symbol.
    #[must_use]
    pub fn component(name: impl Into<Box<str>>) -> Self {
        Self::new(name, SymbolKind::Component)
    }

    /// Returns the symbol's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the symbol's kind.
    #[must_use]
    pub const fn kind(&self) -> &SymbolKind {
        &self.kind
    }

    /// Returns whether this symbol may be read as an expression value (spec
    /// §2.10); see [`SymbolKind::is_reference_target`].
    #[must_use]
    pub const fn is_reference_target(&self) -> bool {
        self.kind.is_reference_target()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_set_name_and_kind() {
        assert_eq!(Symbol::prop("count").name(), "count");
        assert_eq!(Symbol::prop("count").kind(), &SymbolKind::Prop);
        assert_eq!(Symbol::state("open").kind(), &SymbolKind::State);
        assert_eq!(Symbol::loop_local("item").kind(), &SymbolKind::LoopLocal);
        assert_eq!(Symbol::component("Row").kind(), &SymbolKind::Component);
        assert_eq!(
            Symbol::new("f", SymbolKind::Function).kind(),
            &SymbolKind::Function
        );
    }

    #[test]
    fn is_reference_target_only_for_value_bindings() {
        assert!(Symbol::prop("a").is_reference_target());
        assert!(Symbol::state("b").is_reference_target());
        assert!(Symbol::loop_local("item").is_reference_target());
        assert!(!Symbol::component("C").is_reference_target());
        assert!(!Symbol::new("f", SymbolKind::Function).is_reference_target());
        assert!(!Symbol::new("s", SymbolKind::Slot).is_reference_target());
    }
}
