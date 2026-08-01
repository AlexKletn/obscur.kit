Whether a text-changing edit to `field` provably cannot change any retained
box — a perf optimization that stays pixel-identical to the full-frame path.

A **mutating** edit only stays box-stable when the focused field:
* is single-line (not a `<textarea>`) and does **not** wrap — so the line
  count (box height) is pinned at one; and
* hugs the leading-left edge — its `text-align`/`direction` give a paragraph
  alignment offset that is independent of the box width (see
  [`alignment_ignores_width`]) — so the stale (content-sized) text-node box
  width cannot shift the painted run/caret; and
* has a **definite** (non-auto) width — so neither the field's own box nor
  any ancestor resizes with the typed content (a compose-only frame reuses
  every retained box, so nothing may depend on the grown content width).

Any other field (wrapping, multiline, centre/right/RTL-aligned, or
auto/content-sized width) returns `false` and takes a full `Input` frame so
its box relayouts — the measure↔compose box-sync invariant is preserved.
