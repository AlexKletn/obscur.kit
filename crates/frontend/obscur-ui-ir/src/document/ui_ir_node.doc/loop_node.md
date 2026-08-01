Creates a [`Loop`](UiIrNodeKind::Loop) control-flow node with stable `id`
and control keyword `tag` (`"For"` or `"VirtualFor"`, spec §2.11).

The loop's mandatory stable key is attached with [`with_key`](Self::with_key)
and its iterated collection flows in as an `each` property; the repeated
body is its children.
