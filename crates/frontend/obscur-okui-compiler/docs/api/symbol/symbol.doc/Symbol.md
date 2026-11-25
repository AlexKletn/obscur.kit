A named entity visible within a lexical scope (spec §2.10).

A symbol pairs a `name` with its [`SymbolKind`]. Symbols are held by a
[`Scope`](crate::symbol::Scope) keyed by name and looked up outward through
the enclosing scopes during name resolution.
