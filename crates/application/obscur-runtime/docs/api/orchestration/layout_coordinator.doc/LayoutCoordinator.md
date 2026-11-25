Orchestrates layout over a [`RetainedTree`] (spec §3.10, §4.2).

The coordinator is **orchestration only**: it owns the tree walk and the
[`ComputedStyle`](obscur_style::computed::ComputedStyle) → [`LayoutStyle`] mapping, but the flow/flex math lives
entirely in `obscur-layout`'s [`LayoutAlgorithm`](obscur_layout::algorithm::LayoutAlgorithm) implementations
([`FlowLayout`](obscur_layout::algorithm::FlowLayout), [`FlexLayout`](obscur_layout::algorithm::FlexLayout)). Its two responsibilities are:

1. **Style mapping** — [`layout_styles_from`](Self::layout_styles_from)
   derives a per-node [`LayoutStyle`] from each node's [`ComputedStyle`](obscur_style::computed::ComputedStyle).
2. **Recursive driving** — [`layout`](Self::layout) walks the retained tree
   and, for each node, invokes the algorithm selected by its `display`,
   threading a [`ChildMeasurer`](obscur_layout::algorithm::ChildMeasurer) that recursively sizes children, and
   accumulates absolute [`LayoutBox`]es.

The walk is **cycle-safe** (bounded by [`MAX_DEPTH`]) and **panic-free**:
out-of-range child boxes are handled defensively, and no indexing can panic.

The coordinator is a stateless zero-sized value; construct it with
[`new`](Self::new) or [`Default`].
