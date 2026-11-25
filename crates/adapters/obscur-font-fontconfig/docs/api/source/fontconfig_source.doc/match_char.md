Finds a face the host says covers `character` — the query a per-character
fallback needs (spec §5.10, §5.18).

This is `fc-match :charset=<hex>`, which is fontconfig's own coverage
index: asked for `U+232B` it answers with a face that has that glyph
rather than with the default sans. That is the whole value here — an
application's chain is a handful of families, and everything outside them
(CJK, emoji, symbols) otherwise draws nothing.

# The answer is a suggestion, not a guarantee

`fc-match` never fails: given a charset nothing installed covers, it
returns the system default with exit status 0 — a face that does *not*
contain the character. **The caller must verify coverage** before trusting
it; `FontRegistry` re-checks the `cmap` and drops a non-covering answer.
Two independent guards, as with [`find_family`](Self::find_family).
