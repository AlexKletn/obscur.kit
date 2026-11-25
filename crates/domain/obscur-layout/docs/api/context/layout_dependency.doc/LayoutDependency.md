Records what a [`LayoutContext`](super::LayoutContext)'s layout output
depends on, so incremental layout can decide precisely what to recompute
(spec §4.3).

A dependency captures the two inputs to a context's measure:

- whether its measured size depends on the incoming constraints (so a
  **constraint change** forces a re-measure), and
- which children's sizes feed its measure (so a **child size change** in one
  of those children forces a re-measure).

# Status: nothing populates this in production — read it as a contract

This type used to describe itself as "the structured replacement for
scattered ad-hoc dirty flags, which §4.3 explicitly rejects as the
architectural model". That was an over-claim about an unbuilt architecture:
every call that sets a dependency ([`with_constraints_affect_size`], [`add_child`])
is in a `#[cfg(test)]` module, so in production every [`LayoutContext`] carries
the empty [`new`](Self::new) dependency and
[`should_remeasure`](super::LayoutContext::should_remeasure) — along with the
[`LayoutInvalidation`](crate::invalidation::LayoutInvalidation) `note_*` gates
built on it — answers for a graph nothing filled in.

What *is* real is a different mechanism. Layout's incremental half
(`LayoutCoordinator::layout_invalidated`) is driven by a per-node
[`LayoutInvalidation`](crate::invalidation::LayoutInvalidation) and a measure
cache, and it derives its two §4.3 gates without consulting this type at all:
the **downward** gate is exact in the cache key (a child re-measures precisely
when its constraints differ, because a different constraint is a different key),
and the **upward** gate is a whole-ancestor-chain eviction rather than a
dependency-driven early stop. This type is what would make that upward gate
*precise* instead of conservative — the day something populates it.

So: a contract for a consumer that does not exist yet, not shipped behaviour.

[`with_constraints_affect_size`]: Self::with_constraints_affect_size
[`add_child`]: Self::add_child
[`LayoutContext`]: super::LayoutContext
