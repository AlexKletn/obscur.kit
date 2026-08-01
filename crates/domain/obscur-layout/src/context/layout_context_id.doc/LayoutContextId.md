Opaque identifier for a [`LayoutContext`](super::LayoutContext) — an
invalidation boundary (spec §4.2, §4.3).

A layout context is anchored at a node that establishes a boundary; this id
names that context. It is a newtype over an integer handle whose numeric
value carries no meaning outside the subsystem that mints it and must never
be treated as a public schema.
