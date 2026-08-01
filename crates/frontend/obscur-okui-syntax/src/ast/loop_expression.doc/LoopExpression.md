Typed AST wrapper over a [`ForLoop`](SyntaxKind::ForLoop) syntax node — a
`<For>` or `<VirtualFor>` control-flow element (spec §2.11).

A loop is element-shaped, so its tag name, attributes and content are read
through the shared [`Element`] view; this wrapper adds the loop-specific
accessors. The iterated collection is given by an `each={…}` attribute and a
stable identity by a mandatory `key={…}` attribute (§2.11).
[`is_virtual`](Self::is_virtual) distinguishes the virtualized
`<VirtualFor>` form.
