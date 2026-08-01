The pointer as a caret: where a press puts the insertion point, what a drag
selects, and what a *run* of presses means (spec §5.11).

The mock shaper gives every ASCII glyph the same 9.6 px advance, so the caret x
of boundary `i` is exactly `i · ADVANCE` past the text leaf's origin — which
turns "did the click land on the right character?" into arithmetic. One test
walks all six boundaries of "hello" and asserts the click offset is the exact
**inverse** of the render caret x; the others use the same scale to check that a
press between two boundaries snaps to the nearer one, that a press before the
start clamps to 0 and past the last glyph clamps to the end, and that a drag
selects the range from the press *anchor* to the pointer and stops extending the
moment the pointer lifts.

Three tests are about the click *run*. Two presses on one spot inside the
multi-click window select the word; three select the line; and a lone press
collapses to a plain caret, which is the guard that the multi-click work cost
the single click nothing. The fourth condition of a run — that it belongs to the
pointer making it — is here too, and it is the one that broke in practice: with a
single shared history the second finger read the first finger's press and
counted itself as a double click, selecting a word nobody double-tapped. The
*time* half of that condition is asserted in `ui_runtime.rs` instead, because it
names the crate-private window constant.

The last two are the negative space. A click on a non-editable node opens no
session at all, and a `Down`+`Up` on a field resolves **no** activation command —
a field binds `on-input`, and a caret placement is not a button press. Plus one
regression: the platform delivers Space as a *named* key, not a character, so
the editing path has to turn it into an inserted `' '` itself or a focused field
silently swallows every space.
