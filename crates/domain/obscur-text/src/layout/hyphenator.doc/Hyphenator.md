A hyphenation port: given a single word, returns the byte offsets **within**
that word at which a hyphenation break is permitted (spec §5.10).

This is a pure, replaceable seam. The domain never embeds Knuth-Liang
patterns or a language dictionary; a real adapter (behind the xtask boundary)
implements this trait with actual hyphenation tables, while the domain wraps
text through the trait object. It is deliberately **object-safe** so the
runtime can hold a `Box<dyn Hyphenator>`.

# Contract

- The returned offsets are **byte** indices into `word` (UTF-8), each marking
  a position at which `word` may be split (the tail begins at that byte).
- By convention an implementation returns only *interior* break points:
  never `0` (the very start) and never `word.len()` (the very end), and it
  should respect a sensible minimum prefix/suffix so a break does not strand
  a single letter. The domain additionally clamps any offset that is not
  strictly interior, so an over-eager adapter cannot corrupt wrapping.
- The result must be **deterministic**: the same `word` always yields the
  same offsets, so measure and compose agree on the same lines.

Author soft hyphens (`U+00AD`) are handled separately by the wrapper and do
**not** need to be reported here; this port only proposes *algorithmic*
break points inside an otherwise unbroken word.
