Lowers a `grid-column-start` / `-end` / `grid-row-start` / `-end` longhand into
a [`PropertyValue::GridLine`]. The CSS-wide keywords (`inherit` / `initial` /
`unset`) take priority; otherwise the value is a single `<grid-line>` edge.
