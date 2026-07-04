//! `Symbol`: a named entity visible within a lexical scope (spec §2.10).

/// The kind of entity a [`Symbol`] names (spec §2.10).
///
/// The kind determines how a symbol may be used during name resolution. The
/// three *reference targets* — [`Prop`](SymbolKind::Prop),
/// [`State`](SymbolKind::State) and [`LoopLocal`](SymbolKind::LoopLocal) — are
/// the value bindings an interpolated expression is allowed to read; the
/// remaining kinds name declarations (components, functions, slots) that live
/// in a scope but are not themselves read as expression values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    /// A component input prop bound by the caller (spec §2.9).
    Prop,
    /// A piece of component-local reactive state (spec §2.9).
    State,
    /// A loop-local binding introduced by a `<For>` body — `item` or `index`
    /// (spec §2.11).
    LoopLocal,
    /// A component declaration referenced by name (spec §2.9).
    Component,
    /// A callable function declaration (spec §2.10).
    Function,
    /// A named content slot (spec §2.9).
    Slot,
}

impl SymbolKind {
    /// Returns whether a symbol of this kind may be read as an expression value
    /// (spec §2.10).
    ///
    /// Only the value bindings — [`Prop`](SymbolKind::Prop),
    /// [`State`](SymbolKind::State) and [`LoopLocal`](SymbolKind::LoopLocal) —
    /// are valid reference targets; declarations of components, functions and
    /// slots are not.
    #[must_use]
    pub const fn is_reference_target(&self) -> bool {
        matches!(self, Self::Prop | Self::State | Self::LoopLocal)
    }
}

/// A named entity visible within a lexical scope (spec §2.10).
///
/// A symbol pairs a `name` with its [`SymbolKind`]. Symbols are held by a
/// [`Scope`](crate::symbol::Scope) keyed by name and looked up outward through
/// the enclosing scopes during name resolution.
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
