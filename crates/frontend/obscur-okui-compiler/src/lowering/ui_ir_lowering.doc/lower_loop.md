Lowers a `<For>`/`<VirtualFor>` into a [`Loop`](obscur_ui_ir::document::UiIrNodeKind::Loop)
node, carrying the mandatory `key={…}` on the node and the iterated
collection (`each={…}`) as a property; a missing key is a non-fatal
[`LoweringError::MissingLoopKey`] diagnostic (spec §2.11).
