Resolves these insets into the absolute border-box [`Rect`] of an
out-of-flow ([`Position::Absolute`](crate::model::Position::Absolute) /
[`Position::Fixed`](crate::model::Position::Fixed)) node within its
containing block `cb` (spec §4.1).

`width` / `height` are the node's explicit size preferences and `natural`
its measured natural (content) size; each is used when the corresponding
axis is not fully pinned by both insets. Insets and explicit sizes resolve
percentages against `cb`. Per axis (horizontal shown; vertical analogous
with `top` / `bottom` / `height`):

* **both `left` and `right` definite** → the box is pinned to both edges:
  `x = cb.x + left`, `width = cb.width − left − right` (clamped ≥ 0). The
  insets win; the explicit/natural width is ignored.
* **only `left` definite** → `x = cb.x + left`, `width` = the explicit
  width if set, else `natural.width`.
* **only `right` definite** → `width` = explicit-or-natural, then
  `x = cb.right − right − width`.
* **neither definite** → **static-position fallback**: `x = cb.x` (a
  documented simplification — the true static position would be the node's
  in-flow position), `width` = explicit-or-natural.

`box_sizing` and `border_padding` (the per-axis border + padding sum) convert
an explicit `width`/`height` — which under `box-sizing: content-box` is a
*content* size — into the **border-box** size this function returns: the
border box is `content_extent(size) + border_padding`, so a `content-box`
width grows by its border + padding and a `border-box` width is used as-is
(CSS 2.1 §10.3.7 uses content widths; the flow path applies the same
[`BoxSizing::content_extent`]). Insets and `natural` are already border-box
values and are unaffected.
