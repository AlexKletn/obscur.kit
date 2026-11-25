Applies `letter_spacing` (§8.1) and `word_spacing` (§8.2) to already-shaped
`runs` over the source `text`, returning the runs with adjusted advances.

`letter_spacing` is added after each typographic character unit (approximated
by the shaper's cluster grouping); `word_spacing` is added to each word
separator's glyph. Glyph identity, cluster mapping, offsets, direction, font,
bidi level, and synthesis are all preserved — only `x_advance` changes.

A zero/zero call is a no-op and returns `runs` unchanged (byte-identical),
which is what an absent or `normal` `letter-spacing`/`word-spacing` resolves
to, so an unauthored text node is completely unaffected.

Runs travel in and out as the shaping cache's own `Arc<[ShapedRun]>`, so that
common no-op returns the shared slice untouched — no glyph is copied on the
frame path at all. Only a call that actually respaces builds a new slice, and
it must: the cache's copy is shared with every other reader of the same
paragraph and cannot be edited in place.
