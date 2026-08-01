IME composition (spec §5.12): the text a user can see but has not yet typed.

A preedit is displayed and **not** committed, and every one of these five tests
turns on keeping those two apart. Composing "ni" over the value "ab" leaves the
buffer at "ab" while the *displayed* text is "abni" — so the assertions read
both, the buffer through `editing_text` and the display by concatenating the
frame's `DrawText` runs. Only the commit folds "に" into the buffer, clears the
composition and fires `on-input`; a cancel drops the preedit and fires nothing.

The underline test is the one that would catch a splice done in the wrong
coordinate space: the composing glyphs carry a thin (height ≈ 1) fill that must
start exactly two advances past the run origin — where the preedit begins in the
*spliced* text, not in the buffer — and span exactly the two composing glyphs.

The other two are the boundaries. With nothing focused there is no session, so
`Start`, `Update` and `Commit` all answer `false` and no session springs into
being from an IME event alone. And the ordinary keystroke path is proved
untouched: a runtime that runs a whole Start→Cancel cycle before typing 'x'
paints a byte-identical display list to one that just types 'x'.
