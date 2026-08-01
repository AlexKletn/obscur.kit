A parsed `grid-template-areas` grid: the value of a grid container's
`grid-template-areas`. The [`GridAreaTemplate`] carries every named area's
line-number rectangle (already validated as a filled rectangle at parse
time); the runtime looks a child's `grid-area: <name>` up against its
container's template to resolve a concrete placement (spec §4.2).
