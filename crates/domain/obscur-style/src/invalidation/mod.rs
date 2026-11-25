//! Invalidation module.
mod style_dependency;
pub use style_dependency::StyleDependency;
mod style_invalidation;
pub use style_invalidation::StyleInvalidation;
mod style_invalidation_reason;
pub use style_invalidation_reason::StyleInvalidationReason;

// `StyleCache` (and the `CacheEntry` that was only ever its value) memoised one
// `ComputedStyle` per node so an unchanged node could be reused. That reuse is
// real and it ships — but it is the runtime's retained `ComputedStyles` map that
// does it: the style phase moves the previous frame's map out with `mem::take`
// and hands it back as this frame's, resolving only the nodes in the
// `StyleInvalidation` set above, so a node outside that set costs nothing at
// all. Measured, not assumed: a hover on one leaf of a twelve-element tree
// reports `style.resolved == 1` (`obscur-kit/tests/incremental_restyle.rs`).
// A second cache would have had nothing left to save, and its key was the
// weaker one: `get_versioned` compared a `VersionStamp`, and the only
// `VersionStamp` production mints is `RetainedTree::version` — one counter for
// the whole tree, bumped by any structural edit — so every node would have
// missed together where the per-node set above misses one. It comes back only
// if a node ever gets a version of its own AND the retained map stops being the
// thing handed forward.
