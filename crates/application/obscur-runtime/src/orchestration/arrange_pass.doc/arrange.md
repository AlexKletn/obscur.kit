Arrange pass: lays `node` out under `constraints`, writes its **absolute**
[`LayoutBox`] (translated by `origin`) into `boxes`, and recurses into each
child at its accumulated absolute origin.

The algorithm is invoked with a [`RecMeasurer`] that only *measures* (pure),
so `boxes` is **not** borrowed during the call — after the call returns we
are free to mutate `boxes`. This measure/arrange split is what keeps the
recursion borrow-clean: measuring never reads or writes the results map, so
there is no aliasing between the mutable `boxes` borrow held across the
recursion and the shared borrows a measurer needs.

`depth` bounds the recursion: at zero it returns immediately.

Child boxes returned by the algorithm are **parent-relative**; each child's
absolute origin is `origin + child_box.origin()`. The child count and the
returned box count are reconciled defensively so no index can panic.

# Positioning (spec §4.1)

`containing_block` is the absolute rect an [`Position::Absolute`] descendant
resolves its insets against (the padding box of the nearest positioned
ancestor); `viewport` is the fixed-positioning reference for a
[`Position::Fixed`] descendant. A **positioned** node
([`Position::is_positioned`]) establishes a *new* containing block for its
descendants — its own padding box — which is threaded down in place of the
inherited one; a `Static` node is transparent and passes the inherited block
through unchanged. Out-of-flow children (partitioned out of the algorithm's
in-flow list) are positioned after the in-flow walk, each within its
containing block, and then recursed into as an independent positioned root.
