Whether `ch` is a CJK **ideograph** — a large, well-defined set of Unicode
blocks whose characters may line-break between one another by default (UAX #14
class `ID`; CSS Text 3 §5.2 `word-break: normal`).

This is the Han-ideograph subset of UAX#14 class `ID`: the CJK Unified
Ideographs and their extensions plus the compatibility ideographs. Kana and
the CJK symbol/punctuation blocks (which carry additional tailoring rules) are
deliberately excluded so the default breaker stays simple and deterministic;
full UAX#14 tailoring is an adapter concern (see [`LineBreaker`]).
