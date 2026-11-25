Creates a [`Conditional`](UiIrNodeKind::Conditional) control-flow node with
stable `id` and control keyword `tag` (`"If"`, `"ElseIf"` or `"Else"`,
spec §2.11).

The branch condition flows in as a `cond` property (absent on a bare
`<Else>`); the guarded body is its children.
