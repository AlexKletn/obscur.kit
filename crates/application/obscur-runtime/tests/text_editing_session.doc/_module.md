The runtime-owned editing session: what the buffer and caret hold while a field
has focus, and who is allowed to move them.

The session is the sole source of truth for the focused field. That single
sentence is what these twelve tests are about, from three directions.

**The keyboard moves it.** Seeding puts the caret at the end of the value;
arrows move it; Shift extends a selection; Home/End and Ctrl+A reach its edges;
and typing over a selection replaces it. One of these is the sharpest of the
set: a *mutating* edit fires `on-input` with the resulting text and a *caret
move* does not, proven with a sentinel written into the mirror between the two.
And one closes the loop to the screen — after typing, the frame paints the
SESSION's text and a caret at exactly three advances past the run's origin,
while the retained tree still carries the older value. The session override is
the only reason the typed character is visible at all.

**The app may move it, but only from outside.** The controlled-input model
(WHATWG HTML §4.10.5.5) says a text control reflects its `value`, so a rewrite
mid-focus has to reach the display — including a normalizing `on-input` that
uppercases every keystroke. But the *user's own* keystroke, mirrored back
through the app and returned unchanged, must **not** reseed: the third test
edits in the middle of the value on purpose, because a wrongful reseed would
fling the caret to the end, and that is what the user would feel.

**`readonly` moves the line, not the session.** A readonly field still focuses,
still places a caret, still selects all — none of that is gated. What is gated
is mutation, from every entry: typing, Backspace, Delete, single-line Enter and
an IME commit are each inert, no `on-input` fires, and `change` never fires
because the value never changed. The last test is the floor underneath all of
it: with a non-editable focused there is no session at all, and a char press is
routed exactly as it was before text entry existed.
