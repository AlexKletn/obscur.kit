The kind of entity a [`Symbol`] names (spec §2.10).

The kind determines how a symbol may be used during name resolution. The
three *reference targets* — [`Prop`](SymbolKind::Prop),
[`State`](SymbolKind::State) and [`LoopLocal`](SymbolKind::LoopLocal) — are
the value bindings an interpolated expression is allowed to read; the
remaining kinds name declarations (components, functions, slots) that live
in a scope but are not themselves read as expression values.
