Resolves a text node's `word-spacing` into extra per-separator spacing, in
logical pixels, from its computed `word-spacing` (CSS Text 3 §8.2).

Reads [`PropertyId::WordSpacing`] with the same rules as
[`letter_spacing_for`]: a length (possibly negative) resolves verbatim — `px`
verbatim, `em` × `font_size`, `rem` × `root_font_size` (CSS Values 4 §5.1.1);
`normal` / absent / a percentage (CSS Text 3 §8.2 — dropped) / other → `0.0`.
Fed to the shared spacing seam on both the measure and compose passes so they
cannot drift.
