One lexical scope in the scope tree (spec §2.10).

A scope owns the [`Symbol`]s defined directly within it, keyed by name in a
[`BTreeMap`] so iteration and equality are deterministic. `parent` is the
arena index of the enclosing scope inside the owning
[`SymbolTable`](crate::symbol::SymbolTable), or `None` for the root scope.
Resolution of a name that is not defined in this scope walks outward through
the `parent` links (see [`SymbolTable::lookup`](crate::symbol::SymbolTable::lookup)).
