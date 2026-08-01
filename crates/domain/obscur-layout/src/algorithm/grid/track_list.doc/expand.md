Expands this list into a concrete track vector for a `content_extent`-wide
axis with `gap` between tracks.

With no auto-repeat this is simply `leading ++ trailing` and the second
element is [`None`]. With an auto-repeat the pattern is repeated `N` times
(§4.1): `N = max(1, floor((free + gap) / (pattern_extent + gap)))`, where
`free` is the content extent minus the definite (fixed/percentage) size of
the leading and trailing tracks and their gaps, and `pattern_extent` is the
sum of the pattern's definite track sizes plus its interior gaps. The result
is `leading ++ (N × pattern) ++ trailing`.

The second element is `Some((start, end))` — the half-open index range of
the generated pattern tracks — **only** for an `auto-fit` repeat, so the
caller can collapse the empty ones after placement; it is [`None`] for
`auto-fill` (whose empty tracks are kept).
