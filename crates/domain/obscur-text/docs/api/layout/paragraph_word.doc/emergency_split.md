Attempts an **emergency** grapheme-cluster break of this word so its head
fits within `limit`, returning the head glyphs and the remaining tail as a
fresh word (CSS Text 3 §5.2 `overflow-wrap` / `word-break: break-all`).

Unlike [`split_at_hyphen`](Self::split_at_hyphen) this inserts **no** hyphen
glyph — an emergency break is invisible. The largest prefix whose cumulative
advance is `<= limit` is taken, cut only at a **grapheme-cluster boundary**:
glyph clusters are UTF-8 byte offsets, so a boundary sits wherever the
cluster changes and the cut never lands inside a multi-glyph cluster (a base
plus its combining marks, or a ligature). Determinism follows from the greedy
left-to-right scan.

Returns [`None`] when no break is warranted or possible:
- the word is a single glyph (nothing to split);
- no cluster fits `limit` and `line_empty` is false — the caller should
  close the current line and retry on a full-width line;
- the fitting prefix is the whole word (no interior boundary before the
  end) — the caller places it whole.

When `line_empty` is true and not even the first cluster fits `limit`, the
first cluster is forced onto the (fresh) line so it overflows by a single
grapheme rather than looping forever — matching CSS, where one unbreakable
grapheme wider than its box still overflows.
