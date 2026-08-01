The `measure(node, constraints)` results retained across frames — §4.2's
"layout context … may cache measure/layout results", and what makes §3.4
step 8 *incremental*: the frame's [`LayoutInvalidation`] evicts the entries
it can have changed and the rest are served without re-measuring.

Every reuse rests on the preconditions
[`LayoutCoordinator::layout_invalidated`] documents, so anything that
changes a layout input **outside** the computed styles the classifier diffs
must clear it — see the shaping-port setters, which do.
