Appends every recorded bracket to `units` as a `Push*` at the node's enter
index and a `Pop*` at its subtree's *exit* index — the largest enter index
within the subtree — so a pop sorts after every descendant.

The exit index is the running max of each node's own enter index propagated
up to its parent, walking the pre-order sequence in reverse (descendants
before ancestors). `zmap` and `layermap` key each unit exactly like the
node's own paint, so a bracket never separates from the subtree it wraps.

The **emission order between kinds is the nesting order**, because the
stable sort breaks every pop tie at the shared `(z, exit, MAX − depth)` key
in emission order. Reordering these loops reorders the `Pop*`s.
