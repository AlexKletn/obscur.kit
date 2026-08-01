Appends `element`'s members, looking through the control flow that
splices into it (see the [`contract_rule`](crate::validation::contract_rule)
module docs).

`<Empty>` / `<Separator>` inside a `<For>` are looked through for the same
reason: they are §2.11 slots whose *bodies* reach the loop's parent, not
nodes of their own.

The same walk records [`content`](Self::content): a text child or an
`{expr}` reaching this instance, from here or from inside the control flow
that splices into it. Whitespace-only text is not content — the lowering
trims it away and emits no node, so refusing it would reject the
indentation of every multi-line document.
