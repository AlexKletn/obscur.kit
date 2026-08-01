Accumulates the `node id → source span` links discovered while lowering a
`.okui` document, then hands back a finished [`SourceMap`] (spec §2.15).

Each IR node the lowering pass creates is recorded here against the byte
offset it originated from, so diagnostics raised later against the IR can be
mapped back to the source the author wrote.

# Precision

Only the byte **offset** of a node's source is tracked for now: every
recorded span is a zero-width point at that offset (`start == end`) with a
placeholder `line`/`column` of `1`. Resolving offsets to real line/column
positions is a future refinement and does not change the recorded ids.
