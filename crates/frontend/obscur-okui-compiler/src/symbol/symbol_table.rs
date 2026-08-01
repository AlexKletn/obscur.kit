#![doc = include_str!("symbol_table.doc/_module.md")]

use crate::symbol::{Scope, Symbol};

#[doc = include_str!("symbol_table.doc/SymbolTable.md")]
#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    current: usize,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    #[doc = include_str!("symbol_table.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::root()],
            current: 0,
        }
    }

    #[doc = include_str!("symbol_table.doc/push_scope.md")]
    pub fn push_scope(&mut self) -> usize {
        let index = self.scopes.len();
        self.scopes.push(Scope::new(Some(self.current)));
        self.current = index;
        index
    }

    #[doc = include_str!("symbol_table.doc/pop_scope.md")]
    pub fn pop_scope(&mut self) -> bool {
        match self.scopes[self.current].parent() {
            Some(parent) => {
                self.current = parent;
                true
            }
            None => false,
        }
    }

    #[doc = include_str!("symbol_table.doc/current_index.md")]
    #[must_use]
    pub const fn current_index(&self) -> usize {
        self.current
    }

    #[doc = include_str!("symbol_table.doc/define.md")]
    pub fn define(&mut self, symbol: Symbol) -> bool {
        self.scopes[self.current].define(symbol)
    }

    #[doc = include_str!("symbol_table.doc/lookup.md")]
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut cursor = Some(self.current);
        while let Some(index) = cursor {
            let scope = &self.scopes[index];
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
            cursor = scope.parent();
        }
        None
    }

    #[doc = include_str!("symbol_table.doc/is_defined.md")]
    #[must_use]
    pub fn is_defined(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    #[doc = include_str!("symbol_table.doc/scope_count.md")]
    #[must_use]
    pub fn scope_count(&self) -> usize {
        self.scopes.len()
    }

    #[doc = include_str!("symbol_table.doc/enter_root.md")]
    pub const fn enter_root(&mut self) {
        self.current = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::SymbolKind;

    #[test]
    fn new_table_has_one_root_scope() {
        let table = SymbolTable::new();
        assert_eq!(table.scope_count(), 1);
        assert_eq!(table.current_index(), 0);
    }

    #[test]
    fn push_and_pop_move_the_cursor() {
        let mut table = SymbolTable::new();
        let child = table.push_scope();
        assert_eq!(table.current_index(), child);
        assert_eq!(table.scope_count(), 2);
        assert!(table.pop_scope());
        assert_eq!(table.current_index(), 0);
        // Popping at the root fails and leaves the cursor at the root.
        assert!(!table.pop_scope());
        assert_eq!(table.current_index(), 0);
    }

    #[test]
    fn lookup_finds_symbol_in_enclosing_scope() {
        let mut table = SymbolTable::new();
        assert!(table.define(Symbol::prop("count")));
        table.push_scope();
        // Defined in the outer scope, visible from the inner one.
        assert!(table.is_defined("count"));
        assert_eq!(
            table.lookup("count").map(Symbol::kind),
            Some(&SymbolKind::Prop)
        );
        // Inner-scope binding is not visible after popping back out.
        table.define(Symbol::loop_local("item"));
        assert!(table.is_defined("item"));
        table.pop_scope();
        assert!(!table.is_defined("item"));
    }

    #[test]
    fn define_duplicate_in_same_scope_fails() {
        let mut table = SymbolTable::new();
        assert!(table.define(Symbol::prop("a")));
        assert!(!table.define(Symbol::state("a")));
    }

    #[test]
    fn enter_root_resets_cursor() {
        let mut table = SymbolTable::new();
        table.push_scope();
        table.push_scope();
        table.enter_root();
        assert_eq!(table.current_index(), 0);
    }
}
