Groups a non-empty glyph `stream` into per-line [`PackedLine`]s under a
greedy `max_width` fit, breaking an over-wide word at its last fitting
hyphenation opportunity (spec §5.10). `max_width` is assumed positive.

`offsets` holds the full-text byte offsets of the algorithmic and
author-soft-hyphen break points and of the collapsible whitespace, so a
word's trailing space can be excluded from the fit test (it hangs, CSS
Text 3 §3/§7.1); `hyphen_glyph` (when present) is appended at each taken
break so a visible hyphen renders.
