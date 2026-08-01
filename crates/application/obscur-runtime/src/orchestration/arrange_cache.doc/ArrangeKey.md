The exact input tuple one [`arrange`](super::arrange_pass::arrange) call's subtree output is a function of,
reduced to bit patterns so it compares exactly (no hashing, no `f32` `PartialEq`
surprises — a `NaN` inset simply never matches itself, which fails safe by
re-arranging).

# Why these five, and why by value

`arrange(N, …)`'s output depends on (a) the styles/tree of `N`'s subtree, and
(b) where and how big its caller told it to be. A [`LayoutInvalidation`](obscur_layout::invalidation::LayoutInvalidation) bounds
(a) — that is [`invalidated_layout`](super::layout_eviction::invalidated_layout). Nothing bounds (b): a later sibling of a
grown node has no style change and is not dirty, but its `origin` moves. So (b)
is compared by value:

* `origin` — the absolute top-left. A node that moved re-arranges, and so its
  whole subtree does, which is the *only* thing that makes absolute positions
  safe to retain.
* `constraints` — a node re-sized by its parent's algorithm re-arranges.
* `containing_block` — a positioned descendant resolves its insets against it.
* `viewport` — the `position: fixed` reference.
* `scroll` — `position: sticky` clamps against the scrollport.

The remaining inputs (the shaper, the hyphenator, the editing view, the root
`font-size`) are frame-global and *not* keyed on, exactly as [`MeasureCache`](super::measure_cache::MeasureCache)
does not key on them: they ride the caller contract that
[`layout_invalidated`](super::LayoutCoordinator::layout_invalidated) states, and a
dirty root is unboundable. `viewport` and `scroll` are frame-global too and
would ride that contract just as well; they are keyed on because they are the
two arrange reads with no measure counterpart and they cost four words, so the
cache is sound on its own terms rather than only in combination with a
classifier in another module.
