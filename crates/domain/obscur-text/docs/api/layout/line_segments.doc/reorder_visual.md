Reorders a line's logical-order `segments` into visual (left-to-right)
order in place, applying the UAX#9 L2 rule using each segment's bidi
[`level`](ShapedRun::level).

From the highest level present down to the lowest odd level, every maximal
contiguous span of segments whose level is `>= lvl` is reversed. This
places right-to-left runs (and nested higher-level runs within them) into
their correct visual position without touching within-segment glyph order.
A line that is entirely even-levelled (pure LTR) has no odd level and is
left untouched. The reversal is a pure function of the levels, so the
result is deterministic.
