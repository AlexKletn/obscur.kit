Whether this context must be re-measured given a possible constraint
change and/or a changed child (spec §4.3).

Returns `true` when either:
- `constraints_changed` is set and this context's size depends on the
  incoming constraints ("propagate down only on constraint change"), or
- `changed_child` is a child whose size feeds this context's measure.

This encodes that recomputation starts at the nearest affected context
and is driven by structural dependencies rather than dirty flags.
