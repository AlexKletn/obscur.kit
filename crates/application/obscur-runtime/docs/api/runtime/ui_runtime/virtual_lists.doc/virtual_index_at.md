Resolves an on-screen point to the virtual list and model index under it, if
any (spec §4.9).

`screen` is the point in the **unscrolled** layout-box space — the space the
spatial index and every [`LayoutBox`] live in, i.e. the pointer after device
scale and content offset but *before* per-element scroll is added back. The
point must fall within a tracked container's visible box; the container's own
`scroll_top` is then added to turn it into a content-space offset, and the row
extent maps that to an index. A tracked container always has spacers, so its
content is at least a viewport tall and every point in its clamped scrollport
lands on a real row — no separate item-count bound is needed.

[`LayoutBox`]: obscur_layout::model::LayoutBox
