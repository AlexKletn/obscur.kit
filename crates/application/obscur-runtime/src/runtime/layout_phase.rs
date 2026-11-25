use obscur_layout::invalidation::LayoutInvalidation;

#[doc = include_str!("../../docs/api/runtime/layout_classifier.doc/LayoutPhase.md")]
pub(crate) enum LayoutPhase {
    /// No node's box can have moved: the phase is skipped entirely and the
    /// retained boxes + hit-test index carry over.
    Skip,
    /// The frame cannot prove which contexts changed (the tree moved, the viewport
    /// moved, a resource landed, ...), so every node is re-measured and the
    /// retained [`MeasureCache`](crate::orchestration::MeasureCache) is dropped.
    Full,
    /// Exactly these contexts' inputs changed. Every other node's measure is served
    /// from the retained [`MeasureCache`](crate::orchestration::MeasureCache) — see
    /// [`LayoutCoordinator::layout_invalidated`](crate::orchestration::LayoutCoordinator::layout_invalidated)
    /// for what that reuse rests on.
    Invalidated(LayoutInvalidation),
}
