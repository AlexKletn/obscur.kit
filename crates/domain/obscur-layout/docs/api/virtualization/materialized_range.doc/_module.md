The [`MaterializedRange`] — the half-open range of item indices currently
materialized (spec §4.9 *materialization*).

A virtualized collection realizes only a window of its items; this value is
that window, expressed as a half-open `[start, end)` index range. It is the
output of the layout's visible-range computation and the input to whatever
actually builds the item nodes.
