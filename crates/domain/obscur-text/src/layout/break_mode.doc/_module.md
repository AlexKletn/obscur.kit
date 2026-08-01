The `overflow-wrap` and `word-break` line-break controls (CSS Text 3 §5.2).

These two computed values govern whether — and how aggressively — the line
breaker may split *within* an otherwise unbreakable token so it does not
overflow its line. They are the emergency-break companions of the ordinary
(whitespace / UAX#14) break opportunities: the break opportunities say where a
line *may* break between words, while these say whether a word that still does
not fit may be forced apart at a grapheme-cluster boundary.

Both properties inherit and both default to their `normal` value, which keeps
the historical behaviour exactly: an over-wide unbreakable token overflows its
line rather than being split (CSS Text 3 §5.2, which is why an English word
wider than its column pokes out of the box by default).
