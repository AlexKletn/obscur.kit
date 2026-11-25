Derives the semantic view: a [`COMPONENT_ID`] element named `table` and
carrying the sorted-column index (`-1` when unsorted), a header row child
named `tr` (one `th` cell per column, each carrying its title and sortable
flag), then one `tr` child per data row (one `td` cell child per cell).

The names are the point. They are the reserved `LOCAL_NAME` property — an
element's CSS *type* (spec §4.13) — so the UA stylesheet's `table`, `th`,
`td` and `tr:hover` rules reach this widget's own nodes, and reach them by
exactly the same rule that reaches hand-written `<table>` markup. The parts
used to carry a private `role` property instead, which is neither a class nor
a local name and which no selector could ever see.

[`COMPONENT_ID`]: Self::COMPONENT_ID
