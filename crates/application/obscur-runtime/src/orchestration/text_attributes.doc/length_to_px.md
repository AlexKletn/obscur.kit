Resolves a computed text-metric [`Length`] to logical pixels against the
node's font context, per CSS Values 4 §5.1.1: `px` verbatim (byte-identical to
the previous `Length::as_px` path), `em` × the node's own computed
`font-size`, `rem` × the document root's computed `font-size`.

A percentage and the viewport units are **not** resolved here: `%` is
property-specific (font-size-relative for `line-height` — handled by its
caller — and invalid for `letter-spacing` / `word-spacing` / `tab-size`), and
viewport units are a documented refinement, so both return [`None`] and each
caller applies its own fallback (drop-to-`0`, the metric line height, or the
initial tab size).
