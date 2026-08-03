#![doc = include_str!("mod.doc/_module.md")]
mod layout_invalidation;
pub use layout_invalidation::LayoutInvalidation;
mod layout_invalidation_reason;
pub use layout_invalidation_reason::LayoutInvalidationReason;
mod propagation_result;
pub use propagation_result::PropagationResult;

// `LayoutCache` (and the `CacheEntry` that was only ever its value, and the
// `model::LayoutResult` newtype that entry carried) offered a per-node
// measure/arrange cache with a constraint-keyed early exit. Both halves ship —
// as the runtime's `MeasureCache` and `ArrangeCache`, driven by the
// `LayoutInvalidation` set above, and both are better keyed than this was.
// Measured, not assumed: an unchanged frame re-arranges 0 nodes and a dirty
// leaf re-arranges exactly itself and its two ancestors
// (`orchestration/layout_coordinator.rs` tests). So there was no cost left to
// remove, and wiring this one would have been a regression twice over.
// `get_arrange` was keyed on the `NodeId` alone, where an absolute box also
// depends on the origin, the containing block, the viewport and the scroll —
// `ArrangeKey` carries all five, because a node that moved without its own
// constraints changing would otherwise be served a stale box. And one entry per
// node cannot hold a node measured under two constraint sets in the same pass —
// the intrinsic probe and the arrange offer — which `MeasureCache` does by
// putting the constraints in the map key. It comes back only if the layout
// caches ever leave the runtime for the domain, and then with those keys, not
// these.
