Wraps like [`wrap`](Self::wrap) but additionally breaks over-wide words at
the opportunities a `hyphenator` proposes plus every author soft hyphen
(`U+00AD`), optionally rendering a visible `hyphen_glyph` at each break
(spec §5.10).

The break opportunities are [`LineBreak::opportunities`] (whitespace and
mandatory breaks — which delimit words) **plus**, for each word, the
`hyphenator`'s interior byte offsets, **plus** every `U+00AD` in `text` as
a soft break. When the greedy packer meets a word that alone is wider than
the remaining line width, it breaks the word at the **last** hyphenation
opportunity whose head (plus the hyphen glyph's advance) still fits, places
the head — followed by `hyphen_glyph` when supplied — on the line, and
continues the tail on the next line (which may hyphenate again). A word
with no fitting opportunity still takes its own (overflowing) line, exactly
as before. A `U+00AD` that is not taken as a break is left zero-advance and
invisible. Determinism, cluster mapping, bidi reordering, and justification
are preserved: the hyphen glyph carries the break's source cluster and
inherits the font, direction, and bidi level of the glyph it follows, so it
segments and reorders with the head.

[`wrap`](Self::wrap) is the special case `hyphenator = NullHyphenator`,
`hyphen_glyph = None`, and `ellipsis_glyph = None`.

# `text-overflow` truncation

After packing (and justification), any line whose content width *still*
exceeds `max_width` — a single unbreakable, un-hyphenatable word is the
usual cause — is truncated in place when `overflow.is_truncating()` and
`max_width > 0` (spec §5.10). [`TextOverflow::Clip`] keeps the largest
glyph-granular prefix that fits and drops the rest;
[`TextOverflow::Ellipsis`] reserves the `ellipsis_glyph`'s advance, keeps
the largest prefix that fits alongside it, then appends the ellipsis glyph
(carrying the last kept glyph's cluster, font, direction, and level) — with
no `ellipsis_glyph` supplied it degrades to [`Clip`](TextOverflow::Clip).
[`TextOverflow::Visible`] (and any non-truncating policy) leaves overflow
untouched, as does an unconstrained (non-finite `max_width`) wrap. A truncated
line overflows, so justification never stretches it.
