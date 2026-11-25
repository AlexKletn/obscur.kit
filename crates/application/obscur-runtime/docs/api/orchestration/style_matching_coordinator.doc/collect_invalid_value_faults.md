Collects the per-node "invalid value" style faults for this match: for
every element node that matches a rule carrying
[`invalid_values`](obscur_style::matching::StyleRule::invalid_values),
one entry per dropped declaration, attributed to that node (§4.13).

This is the render-phase counterpart of the host-side CSS parse/compile
diagnostics: a `width: not-a-length` never reaches the cascade, but the
node whose matched rule authored it is the one a tool must point at, and
that node is only known here — where selectors meet the tree. Only rules
that actually recorded a dropped value are examined, so a clean sheet
costs one skipped check per rule and produces nothing. Component-internal
nodes are scoped out (an ordinary rule's declarations never reach them),
exactly as [`resolved_for_node`] contains them.
