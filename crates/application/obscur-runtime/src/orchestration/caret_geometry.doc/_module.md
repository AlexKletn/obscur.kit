Where a caret sits in a wrapped paragraph, and which byte a pointer lands on:
the byte ↔ paragraph-geometry mapping the compose pass paints the caret with
and the runtime hit-tests, navigates and scrolls with (spec §5.10, §5.11).

Every function here answers one question about a [`ParagraphLayout`] — the
caret's pen x at a byte, the byte nearest an x, the line a byte sits on — over
the SAME spaced/tabbed glyph advances the segments are painted from, which is
what makes a click resolve back to the offset whose caret was drawn there. The
layout is shaped through the phase's one [`ParagraphShaping`], handed in rather
than held, so paint and hit-test cannot drift onto different engines.
