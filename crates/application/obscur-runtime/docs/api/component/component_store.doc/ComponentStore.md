The registered component kinds plus one live [`MountedComponent`] per node
that instantiates one (spec §2.1, §2.14).

# Why keyed by `NodeId`

A [`SecondaryMap`] *augments* the retained tree's identity rather than
competing with it (spec §1.5): the tree owns and mints the `NodeId`, and this
map attaches the instance to it. That is exactly the relationship wanted here,
and it means [`RetainedNode`](obscur_tree::node::RetainedNode) needs no
component-instance field.

Keying on `NodeId` is also safe against slot reuse: a `NodeId` packs the
arena slot's *generation*, which advances both when a slot is vacated and when
it is reused, so a recycled slot never collides with a stale entry.

The reconciler already resolves `(ComponentId, Option<ViewKey>)`-in-position
into a stable `NodeId` via keyed matching, so the node id *is* the instance
identity — there is no second identity to invent, and an
[`InstanceId`] remains only a diagnostic handle.
