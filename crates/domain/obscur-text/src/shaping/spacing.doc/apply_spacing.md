Applies `letter_spacing` (§8.1) and `word_spacing` (§8.2) to already-shaped
`runs` over the source `text`, returning the runs with adjusted advances.

`letter_spacing` is added after each typographic character unit (approximated
by the shaper's cluster grouping); `word_spacing` is added to each word
separator's glyph. Glyph identity, cluster mapping, offsets, direction, font,
bidi level, and synthesis are all preserved — only `x_advance` changes.

A zero/zero call is a no-op and returns `runs` unchanged (byte-identical),
which is what an absent or `normal` `letter-spacing`/`word-spacing` resolves
to, so an unauthored text node is completely unaffected.
