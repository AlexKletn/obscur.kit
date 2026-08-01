Which formatting context a node establishes, and which children enter it.

Both halves are `display`'s doing (css-display-3), which is why they are one
entity: the keyword picks the [`LayoutAlgorithm`] that will size the box
([`algorithm_for`]), and it also decides who the box's children *are* —
`display: contents` generates no box, so its own children are hoisted into
this formatting context in its place ([`children_of`]), and out-of-flow
children are excluded from it entirely ([`in_flow_children`],
[`partition_children`]).

The measure pass and the arrange pass both walk through here, and they must
agree: the in-flow list's indices are what an algorithm's
[`ChildMeasurer::measure_child`](obscur_layout::algorithm::ChildMeasurer::measure_child)
index means, so a list built one way in one pass and another way in the other
would silently measure the wrong child.
