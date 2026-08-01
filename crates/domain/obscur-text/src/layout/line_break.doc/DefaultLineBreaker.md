The domain-default [`LineBreaker`]: the simplified whitespace / forced-break
opportunities of [`LineBreak::opportunities`], **plus** a soft break between
two adjacent CJK ideographs so unspaced CJK text wraps out of the box (spec
§5.10, UAX #14 class `ID`).

For text containing **no** CJK ideographs — the common Latin / Cyrillic /
whitespace-delimited case — this is *byte-identical* to
[`LineBreak::opportunities`]: the CJK pass adds nothing and the untouched
vector is returned. A host installs a real [`LineBreaker`] adapter for full
UAX#14 tailoring (kana, punctuation, `word-break: keep-all` letter clustering,
…); `keep-all`'s suppression of these inter-ideograph breaks is applied by the
wrapper, so it is honoured even with this default breaker.
