Matches `element` against all `sheets` and returns its cascaded values.

`sheets` must be ordered by ascending cascade origin (system-default
first, inline last); see the type-level docs for how `source_order` is
assigned and why the ordering matters only for intra-band ties. For each
rule whose selector list matches, every authored declaration in its block
becomes a [`Declaration`] carrying the sheet's origin, the highest
matching selector specificity, and the declaration's document position;
the collected declarations are resolved by [`Cascade::resolve`].
