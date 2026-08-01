Creates a **virtual-collection marker**: the node a `<VirtualFor>` lowers
to, carrying the per-row `template` as its children and the estimated row
`row_extent` (logical pixels along the scroll axis) as a reserved property
(spec §2.11, §4.9).

This is the half of a `<VirtualFor>` the front end can build: the window
it stands for is a function of the viewport, which the runtime owns, so
[`realize_virtual_collections`](super::realize_virtual_collections)
replaces this marker with a viewport-bounded window of `template` clones
before the output is reconciled — the marker itself never becomes a
retained node, the same way a slot outlet never does.
