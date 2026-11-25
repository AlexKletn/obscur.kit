Lays out like [`layout_with_hyphenator`](Self::layout_with_hyphenator) but,
while a text-editing session is active, measures the **focused editable's**
text from the live session buffer instead of the (possibly lagging) retained
tree — the SAME string, with the SAME processing, the scene phase composes
(CSS Text 3 §4.1.1).

The retained tree only catches up on the app's reactive rebuild, so between
a keystroke and that rebuild the tree value lags the session by a character.
Sizing the box from the retained value while the scene paints the session
buffer lets the arranged box (line count / width) disagree with the painted
glyphs — worst on a `white-space: normal` / `pre-wrap` field that wraps to a
different number of lines. Threading the same [`EditingView`] into both
passes makes them shape the identical string (raw buffer + the field's real
`white-space`, `process_text` skipped so caret byte offsets stay exact), so
the measured box height equals the painted line count. With `editing` =
[`None`] — or for any text node that is **not** the focused editable's text —
this is byte-for-byte
[`layout_with_hyphenator`](Self::layout_with_hyphenator).
