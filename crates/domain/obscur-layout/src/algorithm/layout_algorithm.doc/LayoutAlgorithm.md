The contract every formatting context (flow, flex, ...) implements to lay out
a single node.

# Measure + arrange in one pass

A `LayoutAlgorithm` performs the §4.2 measure and arrange phases for one node
together and returns both results in a [`LayoutOutput`]: the *measure* result
is the returned [`size`](LayoutOutput::size) (the node's border-box size under
`constraints`), and the *arrange* result is the returned
[`child_boxes`](LayoutOutput::child_boxes) (each child's box in this node's
coordinate space). Framing it as two phases still holds — the algorithm first
determines child and self sizes, then positions the children — but the runtime
consumes them from a single call.

# Decoupled from the tree

An algorithm never walks the retained tree. It is handed the node's own
`style`, the slice of its `children` (identity + style only — see
[`LayoutChild`]), the `constraints` under which it must size itself, and a
[`ChildMeasurer`] it calls to size any child under constraints of its
choosing. The runtime owns traversal and, behind the measurer, decides
whether a child is a text leaf (shaped and wrapped directly, without a box
algorithm) or a nested container (laid out recursively). Either way the
algorithm sees only a [`Size`](obscur_geometry::Size).

# Dispatch

The runtime selects the algorithm from `style.display`:
[`Block`](crate::model::LayoutDisplay::Block) uses
[`FlowLayout`](super::FlowLayout),
[`Flex`](crate::model::LayoutDisplay::Flex) uses
[`FlexLayout`](super::FlexLayout), and
[`None`](crate::model::LayoutDisplay::None) generates no box and is not laid
out at all. Phase 1 ships only flow and flex; grid, positioned, docking, and
custom contexts are deferred.
