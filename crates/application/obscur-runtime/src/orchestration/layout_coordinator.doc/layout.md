Lays out the whole tree under `viewport` and returns each node's absolute
[`LayoutBox`].

Returns an empty map when the tree has no root. Otherwise the root is
arranged at the viewport origin under a tight `viewport` constraint, and
every reachable node's absolute box is written into the result.

`styles` supplies the per-node [`LayoutStyle`] (typically produced by
[`layout_styles_from`](Self::layout_styles_from)); a node missing from
`styles` is laid out with [`LayoutStyle::default`]. `computed` supplies
each node's [`ComputedStyle`](obscur_style::computed::ComputedStyle), from which a text leaf's `font-family`
fallback chain is resolved (via [`families_for`](super::families_for)) — the same chain the
scene phase paints with.

# Text is a constraint-aware leaf (spec §5.10)

A [`NodeKind::Text`](obscur_tree::node::NodeKind::Text) node carries no CSS
width/height, so the measure/arrange pass sizes it itself: it shapes the
content through `paragraph` (the multi-font fallback orchestrator) at the
node's [`font_size`](LayoutStyle::font_size) and **wraps** the resulting
font-tagged runs to the finite `max.width` its container hands down,
returning the wrapped paragraph's [`size`](obscur_text::layout::ParagraphLayout::size) as the
node's border box. A long string in a narrow container therefore wraps to
*more, shorter lines* (a taller box) instead of one wide line. The **same**
`paragraph` orchestrator and size drive the scene phase, which re-wraps each
text node to its arranged box width, so the measured height and the painted
line count always agree.

# Redundant work

The selected algorithm runs once per node during `arrange`, and each run
re-measures that node's children (which recurses). A [`MeasureCache`]
memoizes `measure(node, constraints)`, so a node sized under identical
constraints is measured (and its text shaped) **once per frame** rather than
once per ancestor — the dominant cost on a deep, text-heavy tree (spec
§4.2). Re-measures under *different* constraints (intrinsic vs. final
sizing) are still distinct; collapsing those needs an intrinsic-size cache.

This entry point builds a fresh cache per call, so it re-measures the tree
every frame. To carry the cache across frames and re-measure only what a
[`LayoutInvalidation`] says changed, call
[`layout_invalidated`](Self::layout_invalidated) — which is what the runtime
does.
