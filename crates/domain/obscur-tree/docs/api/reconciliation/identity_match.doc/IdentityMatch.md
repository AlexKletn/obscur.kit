The reconciliation decision for a single desired child.

For every desired child, reconciliation must decide whether to **reuse** an
existing retained node — preserving its stable [`NodeId`] and therefore all
state that hangs off that identity (layout boxes, focus, selection,
animation, caches; spec §2.2) — or to **create** a fresh node. This is the
atomic outcome of that decision; a whole parent's children reconcile to an
ordered list of `IdentityMatch`es aligned with the desired children.

The variant carries no kind information: compatibility (same variant, same
component) has already been checked before a [`IdentityMatch::Reuse`] is
produced, so a reuse always targets a node the desired child can inhabit
in place.
