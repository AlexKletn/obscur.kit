Represents one portal (spec §4.14).

A portal changes where a node is *painted* — its visual parent / stacking
position — while **preserving its logical parent**, and therefore its style
and theme scope. It pairs a **source** node (whose logical position is
unchanged) with a **target** ([`PortalTarget`]) naming the visual
destination host.

# Invariant (spec §4.14)

Applying a portal moves **only** the source node's visual parent to the
target host; its logical parent is left exactly as it was. Because logical
parentage defines style/theme scope, the portalled node keeps inheriting from
its original place in the tree even though it paints under the host.

# Not wired (measured 2026-08-07)

**Nothing in production constructs a `Portal` or calls [`apply`](Self::apply).**
Outside this file the only workspace mentions are a comment in
`obscur-runtime/src/orchestration/layout_eviction.rs:42` and this module's own
unit tests. Three things are missing, and none of them lives here:

1. **A target registry.** [`PortalTarget`] names a live [`NodeId`], but the
   authoring vocabulary — `obscur_primitives::Portal` — carries a target
   *string*. Nothing maps a name to a node, so no caller can build the
   `PortalTarget` this type needs.
2. **A visual-child index.** `apply` records the relationship on the two
   endpoints (`visual_parent` on the source, [`NodeFlags::PORTAL_HOST`] on the
   host) and stops there. [`RetainedNode`] has no visual child *list*, so
   "what is portalled into me" cannot be answered without scanning the tree.
3. **Readers.** `visual_parent()` is written by the reconciler and read by no
   production code at all. Layout takes its children from
   `formatting_context::children_of`, compose walks `child_nodes()`, and the
   §9.9 stacking maps inherit `z` down the same logical links — all of which
   would have to consult the visual view instead.

[`RetainedNode`]: crate::node::RetainedNode
[`NodeFlags::PORTAL_HOST`]: crate::node::NodeFlags::PORTAL_HOST

See `docs/api/12-reality.md`, row *Portals*, for the full accounting —
including the one consumer that cannot be fixed from outside
`obscur-runtime/src/runtime/hit_testing.rs`.
