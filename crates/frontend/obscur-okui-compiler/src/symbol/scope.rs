#![doc = include_str!("scope.doc/_module.md")]

use std::collections::BTreeMap;

use crate::symbol::Symbol;

#[doc = include_str!("scope.doc/Scope.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    parent: Option<usize>,
    symbols: BTreeMap<Box<str>, Symbol>,
}

impl Scope {
    #[doc = include_str!("scope.doc/new.md")]
    #[must_use]
    pub fn new(parent: Option<usize>) -> Self {
        Self {
            parent,
            symbols: BTreeMap::new(),
        }
    }

    #[doc = include_str!("scope.doc/root.md")]
    #[must_use]
    pub fn root() -> Self {
        Self::new(None)
    }

    #[doc = include_str!("scope.doc/define.md")]
    pub fn define(&mut self, symbol: Symbol) -> bool {
        if self.symbols.contains_key(symbol.name()) {
            return false;
        }
        self.symbols.insert(symbol.name().into(), symbol);
        true
    }

    #[doc = include_str!("scope.doc/get.md")]
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    #[doc = include_str!("scope.doc/parent.md")]
    #[must_use]
    pub const fn parent(&self) -> Option<usize> {
        self.parent
    }

    #[doc = include_str!("scope.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    #[doc = include_str!("scope.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    #[doc = include_str!("scope.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    #[doc = include_str!("scope.doc/symbols.md")]
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
