Per-element vertical scroll containers: which nodes scroll, their scrollport,
and their scrollable content extent.

Reimplemented from **CSS Overflow Module Level 3** — a box whose computed
`overflow` is `scroll` or `auto` is a **scroll container** (§overflow) with a
**scrollport** (the visible cutout its content is clipped and scrolled within,
§scrollable-overflow) and a **scrollable overflow region** (the union of its
content's overflow, §scrollable-overflow-region). This module is pure and
IO-free: the same tree/style/box inputs always yield the same containers, so
both the runtime (which owns the live scroll position + wheel + hit-test) and
the scene coordinator (which paints the scroll offset + scrollbar) resolve the
identical set from one source of truth.
