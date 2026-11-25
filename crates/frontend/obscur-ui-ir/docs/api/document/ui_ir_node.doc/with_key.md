Attaches the stable key expression `key` to this node, returning the
updated node.

A [`Loop`](UiIrNodeKind::Loop) carries the mandatory `key={…}` of a
`<For>` (spec §2.11) so reconciliation can preserve per-item identity; any
keyed node may carry one.
