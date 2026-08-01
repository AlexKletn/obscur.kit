An **invalidation boundary** anchored at a node (spec §4.2, §4.3).

A layout context owns the child constraints and dependencies for its
subtree, forms an invalidation boundary, and can stop propagation when
geometry remains unchanged. What it can be invalidated by is recorded
structurally in its [`LayoutDependency`], and whether inner changes escape is
recorded in its [`LayoutContainment`] (§4.3).

# Status: nothing constructs one of these in production

Read the paragraph above as this type's contract, not as a description of the
shipped layout phase. [`LayoutContext::new`](Self::new) has no caller outside
`#[cfg(test)]` modules, so no such boundary exists at runtime and neither
[`containment`](Self::containment) nor
[`should_remeasure`](Self::should_remeasure) is ever consulted on a real frame.

The incremental layout that *does* ship takes a different route to the same
§4.3 rules: `LayoutCoordinator::layout_invalidated` gates re-measurement on a
per-node [`LayoutInvalidation`](crate::invalidation::LayoutInvalidation) plus a
measure cache, deriving the downward rule from the cache key (a child
re-measures precisely when its constraints differ) and the upward rule from
whole-ancestor-chain eviction. So "recalculation starts at the nearest affected
context" is not how the runtime currently gets there, and the containment stop
is unimplemented: nothing narrows that eviction. This type is what would make
both precise rather than conservative.
