A generational arena that owns every [`RetainedNode`] in a retained tree.

This is the **single owner of UI identity** (spec §1.5): it is the only thing
that mints [`NodeId`]s, and every node record lives here. Layout boxes, paint
data, accessibility nodes and backend handles do **not** live here — they live
in [`SecondaryMap`](super::SecondaryMap)s keyed by the `NodeId` this arena
mints, and never replace that identity.

Internally the arena is a `Vec` of slots plus a free-list of vacated indices.
Each minted `NodeId` packs the slot index together with that slot's
generation (see [`pack_node_id`]); on every access the encoded generation is
validated against the slot's current generation, so a stale `NodeId` whose
slot has since been reused resolves to `None` rather than aliasing a
different node.

This Phase-1 implementation is a self-contained `std` arena. It is designed to
be swappable for a `slotmap`-backed store later behind this same public API;
the generational packing is deliberately kept private so that swap is
invisible to callers.
