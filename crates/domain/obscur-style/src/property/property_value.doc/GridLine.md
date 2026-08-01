A single grid line placement edge: the value of one `grid-column-start` /
`grid-column-end` / `grid-row-start` / `grid-row-end` longhand. The
[`GridLineValue`] is a style-local line edge (`auto`, a 1-based line index,
or `span N`); the runtime converts it to `obscur_layout`'s `GridLine` and
assembles the four edges into a `GridPlacement` when it bridges the computed
style into a grid layout input (spec §4.13).
