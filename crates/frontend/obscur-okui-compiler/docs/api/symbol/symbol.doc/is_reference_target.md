Returns whether a symbol of this kind may be read as an expression value
(spec §2.10).

Only the value bindings — [`Prop`](SymbolKind::Prop),
[`State`](SymbolKind::State) and [`LoopLocal`](SymbolKind::LoopLocal) —
are valid reference targets; declarations of components, functions and
slots are not.
