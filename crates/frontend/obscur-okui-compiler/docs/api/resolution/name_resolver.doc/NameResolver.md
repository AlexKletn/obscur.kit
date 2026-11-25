Resolves the identifiers referenced by the expressions in a [`Document`]
against a [`SymbolTable`] (spec §2.10).

The caller pre-seeds the table with the component's props and state (via
[`SymbolTable::define`]); [`resolve`](Self::resolve) then walks the element
tree, checking that every attribute expression's leading reference resolves
to a defined symbol.

## What is not a value reference

An `on:<event>` binding is skipped. Its value is a **handler name** that the
lowering hashes into a command symbol, not an expression anything evaluates, so
a symbol table of props and state has no opinion worth reporting about it — see
the note in `resolve_element`. Those names are checked by
[`CommandValidator`](crate::validation::CommandValidator) against the commands
the caller declared (spec §2.13).

## Scoping conventions

A `<For>` loop opens a child scope for its body and binds two loop-local
symbols — `item` (the current element) and `index` (its position) — so
expressions inside the body may read them (spec §2.11). A `<For>` must carry
a `key` attribute for stable per-item identity; a missing key is reported as
[`ResolutionError::missing_loop_key`]. Conditionals (`<If>`/`<ElseIf>`/
`<Else>`) do not open a new scope — their body resolves in the enclosing
scope.
