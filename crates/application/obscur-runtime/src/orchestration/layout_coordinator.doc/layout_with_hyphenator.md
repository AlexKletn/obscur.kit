Lays out like [`layout`](Self::layout) but wraps text-leaf nodes through an
optional `hyphenator`, so an over-wide word breaks at a valid point
(spec §5.10).

The runtime passes the **same** hyphenator (and shapes the **same** hyphen
glyph) the scene phase composes with, so the measured box height and the
painted line count agree even when hyphenation splits a word. [`None`]
reproduces [`layout`](Self::layout) exactly.

# Scroll offset and `position: sticky`

`scroll_offset` is the test-drivable viewport scroll `(x, y)` in logical
pixels: the document-coordinate point currently at the top-left of the
viewport. The single scroll container is the **viewport** (per-container
overflow scroll containers are a follow-up); live scroll-gesture wiring is
separate and simply feeds this value. It is consumed only by
[`Position::Sticky`](obscur_layout::model::Position::Sticky) nodes, whose used position is clamped to the scrollport
edge within their containing block. With `scroll_offset == (0.0, 0.0)` (and
a sticky node still below its inset threshold) every node's box is
**byte-identical** to the pre-sticky walk.
