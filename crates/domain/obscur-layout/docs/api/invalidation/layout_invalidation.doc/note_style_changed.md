Invalidates `node` with
[`StyleChanged`](LayoutInvalidationReason::StyleChanged).

A change to a node's own layout-affecting style always requires
re-laying-out that node, so this is unconditional.
