A grid container's `grid-template-areas` value: a rectangular grid of cells,
each either a named area or a null (`.`) cell, with every named area resolved
to the line-number rectangle it spans (spec §4.2, the named-areas subset).

The template is built from the authored rows of cell tokens via
[`from_rows`](GridAreaTemplate::from_rows), which enforces the two CSS
well-formedness rules:

* every row has the same number of columns, and
* every named area forms a *filled rectangle* (all its cells are contiguous,
  with no hole and no second disjoint block of the same name).

A template that breaks either rule is rejected (`from_rows` returns [`None`]),
so the CSS layer can emit a diagnostic and skip the declaration rather than
laying out against a malformed grid.

The resolved areas are stored **sorted by name**, so the template is a
deterministic value with a stable [`PartialEq`] independent of author order.

# Line numbers

[`area`](GridAreaTemplate::area) returns each area's [`GridAreaRect`] in
1-based grid line numbers, directly usable as a child's `grid-row` /
`grid-column` line placement.
