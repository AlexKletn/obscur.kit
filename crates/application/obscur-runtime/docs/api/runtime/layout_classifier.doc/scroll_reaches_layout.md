Whether the scroll offset can change any box in this document — that is,
whether **any** node is `position: sticky`.

Scroll reaches the layout pass through exactly one door. `arrange` takes
`scroll_offset` only to build the scrollport rect
(`viewport.origin + scroll_offset`), and the only thing that rect is used
for is
[`Inset::sticky_offset`](obscur_layout::model::Inset::sticky_offset) on a
child whose `position` is `sticky` (CSS Position 3 §sticky positioning).
Every other box's origin and size is computed in document coordinates and
does not read the offset at all; the scroll translation is applied later, by
the compose pass (from the live `ScrollOffsets`) and by
`refresh_scroll_folded_boxes` for the public `layout_boxes` accessor.

So a document with no sticky box produces **byte-identical** layout boxes at
every scroll offset, and a scroll-only frame can keep the retained boxes,
the [`MeasureCache`](crate::orchestration::MeasureCache) and the
[`ArrangeCache`](crate::orchestration::ArrangeCache) instead of re-measuring
and re-arranging the tree. That is the difference between a scrolled frame
costing a full re-shape of every visible text leaf and costing nothing at
all — the load-bearing case being a long virtualized list, where the text
leaves dominate the pass.

When a sticky box *is* present the classifier keeps saying
[`Full`](LayoutPhase::Full): the offset genuinely moves that box (and its
subtree), and this function deliberately does not try to bound *which*
subtree, because a sticky box's clamp also depends on its containing
block's arranged geometry.

Linear in the styled node count, with one property probe per node — three
orders of magnitude cheaper than the layout pass it can cancel.
