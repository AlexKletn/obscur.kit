Text that occupies more than one line, and everything that stops being a scalar
when it does (CSS Text 3 §4.1.1, CSS Overflow 3 §3.1).

A single-line field has a caret at *an* x. A multi-line one has a caret on a
*line*, at an x measured within that line — and the bug this file exists to keep
dead is the one where those two are confused: a caret placed at the summed width
of every line, which on line 2 of "ab\ncd" would land four advances out instead
of two. Three of the nine tests are about that arithmetic from three sides —
where `Enter` puts the caret, where `ArrowUp`/`ArrowDown` take it, and where a
*click* in the second line's y-band resolves to. The 2D hit-test picks the line
by y and then the byte by x, so a click can never land on the wrong line.

The gate is `type=textarea`, and one test holds it: the same `Enter` in a
single-line `input` inserts nothing, because there it belongs to the
activation/submit path.

Three more make the box a scroll container. Six typed lines in a three-line box
push `scroll_top` to its maximum and keep the caret's line inside the visible
band — while the *unscrolled* caret is proved to sit below the box, which is
what makes the scroll transform load-bearing rather than decorative. Content
that fits does not scroll at all, and the wheel moves the very same `scroll_top`
the caret follow drives, so the two mechanisms cannot drift apart.

The last one is the invariant underneath all of them: while a field is focused,
its box is SIZED and PAINTED from the same session buffer. Between a keystroke
and the app's reactive rebuild the retained tree lags — here it stays empty — so
a measure pass reading the *retained* value would size a one-line box while
compose painted four. The assertion is deliberately shaped so it can only pass
for the right reason: measured height equals the painted line **count** times
the line height, and exceeds the stale box.
