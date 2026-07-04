//! `SymbolTable`: an arena of lexical scopes with a current-scope cursor
//! (spec §2.10).

use crate::symbol::{Scope, Symbol};

/// An arena of lexical [`Scope`]s with a current-scope cursor (spec §2.10).
///
/// Scopes are stored contiguously in `scopes`; index `0` is always the root.
/// Each non-root scope records the arena index of its parent, so the scope tree
/// is encoded by those parent links. The `current` cursor names the scope that
/// [`define`](Self::define) writes into and that [`lookup`](Self::lookup) starts
/// from.
///
/// Lexical resolution walks outward: [`lookup`](Self::lookup) searches the
/// current scope, then its parent, and so on to the root, returning the first
/// match. This lets an inner scope (for example a `<For>` body) shadow and
/// extend the bindings of its enclosing scopes.
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
    /// Creates a table with a single empty root scope, with the cursor at the
    /// root (index `0`).
    #[must_use]
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::root()],
            current: 0,
        }
    }

    /// Creates a child of the current scope, moves the cursor into it, and
    /// returns its arena index.
    pub fn push_scope(&mut self) -> usize {
        let index = self.scopes.len();
        self.scopes.push(Scope::new(Some(self.current)));
        self.current = index;
        index
    }

    /// Moves the cursor to the current scope's parent, returning `false` (and
    /// staying put) if already at the root.
    pub fn pop_scope(&mut self) -> bool {
        match self.scopes[self.current].parent() {
            Some(parent) => {
                self.current = parent;
                true
            }
            None => false,
        }
    }

    /// Returns the arena index of the current scope.
    #[must_use]
    pub const fn current_index(&self) -> usize {
        self.current
    }

    /// Defines `symbol` in the current scope, returning `false` on a
    /// duplicate-in-scope (see [`Scope::define`]).
    pub fn define(&mut self, symbol: Symbol) -> bool {
        self.scopes[self.current].define(symbol)
    }

    /// Looks up `name` lexically: searches the current scope, then walks the
    /// `parent` links outward to the root, returning the first match.
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

    /// Returns whether `name` resolves to a symbol in the current scope or any
    /// enclosing scope.
    #[must_use]
    pub fn is_defined(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Returns the total number of scopes in the arena.
    #[must_use]
    pub fn scope_count(&self) -> usize {
        self.scopes.len()
    }

    /// Resets the cursor to the root scope (index `0`).
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
        assert_eq!(table.lookup("count").map(Symbol::kind), Some(&SymbolKind::Prop));
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
