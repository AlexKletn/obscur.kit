//! `Scope`: one lexical scope in the scope tree (spec §2.10).

use std::collections::BTreeMap;

use crate::symbol::Symbol;

#[doc = include_str!("../../docs/api/symbol/scope.doc/Scope.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    parent: Option<usize>,
    symbols: BTreeMap<Box<str>, Symbol>,
}

impl Scope {
    /// Creates an empty scope whose enclosing scope is at arena index `parent`
    /// (`None` for the root).
    #[must_use]
    pub fn new(parent: Option<usize>) -> Self {
        Self {
            parent,
            symbols: BTreeMap::new(),
        }
    }

    /// Creates an empty root scope (no parent).
    #[must_use]
    pub fn root() -> Self {
        Self::new(None)
    }

    /// Defines `symbol` in this scope, returning `false` if a symbol with the
    /// same name was already defined here (a duplicate; the existing binding is
    /// kept).
    pub fn define(&mut self, symbol: Symbol) -> bool {
        if self.symbols.contains_key(symbol.name()) {
            return false;
        }
        self.symbols.insert(symbol.name().into(), symbol);
        true
    }

    /// Returns the symbol named `name` defined directly in this scope, if any
    /// (this scope only — no outward walk).
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Returns the arena index of this scope's parent, or `None` for the root.
    #[must_use]
    pub const fn parent(&self) -> Option<usize> {
        self.parent
    }

    /// Returns whether a symbol named `name` is defined directly in this scope.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Returns the number of symbols defined directly in this scope.
    #[must_use]
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Returns whether this scope defines no symbols.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Iterates the symbols defined directly in this scope, in name order.
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_has_no_parent() {
        let scope = Scope::root();
        assert_eq!(scope.parent(), None);
        assert!(scope.is_empty());
    }

    #[test]
    fn define_rejects_duplicates_in_scope() {
        let mut scope = Scope::new(Some(0));
        assert!(scope.define(Symbol::prop("count")));
        assert!(!scope.define(Symbol::state("count")));
        // The original binding is kept.
        assert_eq!(scope.get("count"), Some(&Symbol::prop("count")));
    }

    #[test]
    fn get_and_contains_report_local_symbols() {
        let mut scope = Scope::new(Some(3));
        scope.define(Symbol::prop("a"));
        assert_eq!(scope.parent(), Some(3));
        assert!(scope.contains("a"));
        assert!(!scope.contains("b"));
        assert_eq!(scope.get("a"), Some(&Symbol::prop("a")));
        assert_eq!(scope.get("b"), None);
        assert_eq!(scope.len(), 1);
    }
}
