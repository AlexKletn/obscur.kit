The install must replace the single-run itemizer — the audit's "**no bidi by
default**", which is not merely a missing feature but an actively wrong answer.

`direction: rtl` over `abc.` is the smallest case that shows it. UAX#9 says the
Latin letters are a left-to-right island (level 2) inside an RTL paragraph,
while the trailing `.` is a neutral that takes the paragraph's own level (1) by
rule N2 — so the correct visual order is `.abc`: the period jumps to the left,
the letters do **not** reverse.

Out of the box the whole node is one level-0 run and the renderer reverses it
wholesale to `.cba`, mangling the word. One install call is the difference.

# Why a Latin-only font can test bidi

Neither vendored face covers any RTL script, so this cannot use real RTL
letters. It does not need to: bidi *levels* are what split and reorder
segments, and a neutral beside an L run in an RTL paragraph produces two levels
out of Latin alone. Genuine RTL *shaping* remains untested — no RTL face is
vendored.
