Stretches every justification-eligible line so it fills `max_width`,
distributing the extra space equally across the line's inter-word gaps
(spec §5.10).

A line is **not** justified — it keeps its natural width — when it is the
paragraph's last line, when it was closed by a mandatory (hard) break, or
when it holds fewer than two words (no inter-word gap to stretch). For an
eligible line the extra space `max_width - line_width` is split evenly
across its `g` gaps and added to the trailing glyph of each gap (typically
the word-final space); glyph ids and clusters are left intact, so the line's
rendered width becomes exactly `max_width`. A line already at or past
`max_width` gains nothing.
