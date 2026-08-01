A node whose ONLY matching rule is the `:hover` one must lose its whole
cascade when the hover ends — not merely have it overwritten.

Distinct from [`un_hovering_resolves_the_node_back`], where the node also
matches a plain `.row` rule that re-cascades over the stale entry and hides the
bug. Here nothing matches once the hover ends, so an incremental re-match that
only ever *writes* entries leaves the `:hover` background applied forever.
