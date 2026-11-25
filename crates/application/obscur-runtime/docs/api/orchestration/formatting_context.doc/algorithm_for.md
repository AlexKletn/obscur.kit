Selects the [`LayoutAlgorithm`] for `node`'s `display` mode, or [`None`] when
the node generates no box ([`LayoutDisplay::None`]).

A [`LayoutDisplay::Grid`] node that authored a template (present in
`grid_templates`) dispatches the real track-sizing algorithm via
[`GridLayout::with_template`], with the node's `gap` supplying both the column
and row gap; a grid node with no authored template keeps the default single
implicit column ([`GridLayout::new(1)`]) — spec-faithful when
`grid-template-columns` is unspecified. The returned trait object is a
stateless algorithm; the caller invokes it with the node's style, children,
constraints, and a measurer.
