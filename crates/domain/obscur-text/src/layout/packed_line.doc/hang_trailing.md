Hangs this line's trailing content so it is excluded from the line's used
width, alignment offset, and justification base (CSS Text 3 §7.1, §8.1/§8.2).

Two trims, in order:

1. **Trailing collapsible whitespace (§3/§7.1):** the maximal run of
   collapsible-space glyphs (those `offsets` says
   [`hangs`](BreakOffsets::hangs)) at the end of the line has its advances
   zeroed — a sequence of collapsible spaces at a line's end is *removed*. A
   trailing `word-spacing` folded onto that space is removed with it.
2. **Trailing tracking (§8.1/§8.2):** the last *remaining* cluster carries
   the `letter_spacing` that was added after every cluster; that trailing
   tracking hangs, so it is subtracted (plus `word_spacing` when that last
   cluster is itself a word separator, e.g. a preserved NBSP). `0.0`
   tracking makes this a no-op.

Zeroing (rather than deleting) the glyphs keeps every cluster present for
caret hit-testing; only advances change.
