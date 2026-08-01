Whether the wrapped-line boundary at byte `index` follows a **forced** break —
a mandatory line-break character consumed by the earlier line (`\n`, `\r`, and
the other Unicode segment-break characters, CSS Text 3 §4.1) — rather than a
**soft** wrap where no character is consumed.

Caret [`CaretAffinity`] only disambiguates SOFT boundaries: at a forced break
the two visual positions straddle the break character and are therefore
*different* bytes, never the same offset, so affinity is inert. The soft case
(this returns `false`) is the one where `index` is simultaneously the end of one
line and the start of the next with the preceding character being ordinary
content. A caret offset is always a char boundary; a non-boundary or
out-of-range `index` conservatively reports `false`.
