Wraps like [`wrap_hyphenated`](Self::wrap_hyphenated) but sizes each line
box from the mixed-font line-box **strut** (CSS 2.1 §10.8 / §10.8.1).

For each packed line the used ascent `A = max(strut.ascent, max over
segments of seg_metrics(font).0)` and descent `D = max(strut.descent, max
over segments of seg_metrics(font).1)`; the strut floor is applied even on
an empty line. The content box is `A + D`; when the used `line-height` L
(`strut.line_height()`) exceeds it the excess splits as half-leading
`(L − (A+D))/2` above and below (line box `height = L`,
`baseline = A + half_leading`), else `height = A + D`, `baseline = A`. Lines
stack by their own heights, so a tall fallback line pushes the rest down and
the paragraph height is `Σ line heights`. Packing, hyphenation,
justification, and `text-overflow` are identical to
[`wrap_hyphenated`](Self::wrap_hyphenated).

`tab_width` is the resolved `tab-size` stop width; after packing, each
preserved tab is re-expanded from the start of *its own line* (CSS Text 3
§4.3, see [`wrap_core`](Self::wrap_core)). `0.0` (or text with no `\t`)
disables the pass and is byte-identical, as is a tab that stays on line 0.

`overflow_wrap` / `word_break` are the resolved CSS Text 3 §5.2 controls
for the emergency (grapheme-cluster) break of an over-wide token, and
`line_breaker` is the replaceable [`LineBreaker`] port that supplies the
ordinary break opportunities. The defaults
([`OverflowWrap::Normal`] + [`WordBreak::Normal`] +
[`DefaultLineBreaker`]) keep the historical behaviour exactly — an over-wide
unbreakable word overflows and no port is consulted beyond the built-in
breaker.
