Supplies the computed-value step with a real `0` advance:
[`ShapedChMetrics`].

This is the runtime's implementation of `obscur-style`'s [`ChMetrics`] port —
the adapter that lets the resolver *ask* for the one number it cannot compute.
It shapes the `0` glyph through the same [`ParagraphShaper`] the measure and
compose passes use, so a `20ch` box, the twenty characters placed in it, and
the pixels painted for both are all measured against one font.

# Why this exists at all

`ch` was the last relative unit to die at the layout↔paint seam. Every other
unit folds to `px` at computed-value time, so paint reads pixels; `ch` stayed
symbolic on the `ComputedStyle` because the resolver had no shaper. Layout
could measure it and did, paint could not and read `0` — so
`border-width: 1ch` reserved a 24px ring and painted nothing. Handing the
resolver this port moves the measurement to the same place every other unit is
resolved, which is what makes the two phases agree.
