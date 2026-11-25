Routes a single [`Invalidation`] to the domain accumulators and flags the
dirty pipeline phases.

From the invalidation's [`InvalidationKind`](crate::invalidation::InvalidationKind):
a restyle-needing change flags the style phase; a relayout-needing change
flags the layout phase; every change flags the scene (recompose) phase; a
repaint-needing change flags the paint phase.

When the invalidation carries a node (see [`Invalidation::node`]) the
change is forwarded to the domains: if it needs a restyle and maps to a
concrete [`StyleInvalidationReason`] (see [`style_reason`]) the node is
invalidated in `style`; if it needs a relayout the node is invalidated in
`layout` with the mapped [`LayoutInvalidationReason`] (see [`layout_reason`]).
A global change (node `None`) only sets the phase flags — the frame
pipeline invalidates the whole tree.
