Lowers a parsed `.okui` [`Document`] into a versioned [`UiIrDocument`]
(spec §2.15).

# Algorithm

The pass walks the top-level [`elements`](Document::elements) of the
document and turns each into a [`UiIrNode`]:

- **Root shape.** A document with exactly one top-level element lowers that
  element as the root; several top-level elements are wrapped in a
  [`fragment`](UiIrNode::fragment) root, each lowered as a child. An empty
  document is a [`LoweringError::EmptyDocument`].
- **Node kind.** An element whose tag starts with an uppercase letter becomes
  a [`component`](UiIrNode::component); otherwise an
  [`element`](UiIrNode::element). The element-shaped
  [`Conditional`](Element::is_conditional) (`<If>`/`<ElseIf>`/`<Else>`) and
  [`loop`](Element::is_loop) (`<For>`/`<VirtualFor>`) constructs lower to the
  dedicated [`Conditional`](UiIrNode::conditional)/[`Loop`](UiIrNode::loop_node)
  IR kinds (spec §2.11): a loop carries its mandatory `key={…}` on the node
  (a missing key is a [`LoweringError::MissingLoopKey`] diagnostic) and its
  `each={…}` collection as a property; a conditional carries its `cond={…}`
  as a property.
- **Attributes.** An event attribute (`on:click=…`) becomes a
  [`UiIrEvent`] wiring the local event name (after the `:`) to its handler;
  every other attribute becomes a [`UiIrProperty`] whose value is the lowered
  expression, string literal, or `Null`. A `requires="cap"` attribute is
  additionally collected as a declared capability (the §2.13 convention, as
  in `CapabilityValidator`).
- **Children.** Element/conditional/loop children recurse; a non-blank
  [`TextNode`](SyntaxKind::TextNode) becomes a [`text`](UiIrNode::text) child
  (blank text is skipped); an [`Expression`](SyntaxKind::Expression)
  interpolation becomes a synthetic `Expression` element child carrying the
  lowered expression under a `value` property.
- **Ids and source map.** Every node is assigned a stable id from a counter
  starting at `1`, and each id is recorded against its source byte offset in
  the [`SourceMapBuilder`].
