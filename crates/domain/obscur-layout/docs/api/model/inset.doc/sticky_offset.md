Computes the used-position offset `(dx, dy)` of an **in-flow**
[`Position::Sticky`](crate::model::Position::Sticky) node relative to its
static (normal-flow) position (spec §4.1; CSS `position: sticky`).

A sticky node keeps its flow box (its siblings are never perturbed), but
once its scroll container has scrolled past its inset threshold the node's
*used* position sticks to the scrollport edge, clamped to stay within its
containing block. This returns the delta to add to the node's static
origin — `(0.0, 0.0)` whenever the node is at (or before) its threshold, so
a zero scroll offset is transparent.

* `static_pos` is the node's static absolute origin (top-left).
* `node_size` is its border-box size.
* `cb` is its containing block (the clamp keeps the node inside it).
* `scrollport` is the scroll container's visible rect in the same document
  coordinates as the boxes — its top-left is the viewport origin translated
  by the current scroll offset (`viewport.origin + scroll_offset`), so
  `scrollport.top` is the document-coordinate y currently at the top of the
  viewport. The single scroll container is the viewport.

Each axis is clamped independently. On the vertical axis (horizontal is
analogous with `left` / `right` / width):

* **`top` set** — `sticky_lower = scrollport.top + top`,
  `sticky_upper = cb.bottom − height`;
  `used_top = min(max(sticky_lower, static_top), sticky_upper)` — the node
  never rises above its static position and never falls past the CB.
* **only `bottom` set** — the sticky edge is the scrollport bottom:
  `sticky_from_bottom = scrollport.bottom − bottom − height`;
  `used_top = max(min(sticky_from_bottom, static_top), cb.top)`.
* **neither inset on the axis** — that axis is not sticky (delta `0`).

`top` wins over `bottom` and `left` over `right` when both are set.
