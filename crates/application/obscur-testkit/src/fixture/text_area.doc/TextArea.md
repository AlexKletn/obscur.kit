A `<textarea>`: an editable node carrying the reserved CSS type `textarea`, so
the runtime treats it MULTILINE — `Enter` inserts `\n`, `ArrowUp`/`ArrowDown`
move between lines, and the caret is placed within its own line rather than at
the summed width of all of them.

The sheet is deliberately over-wide (300 px) so that newline-split lines never
*also* wrap: a test that asserts "the caret is on line 2" must be measuring the
authored `\n`, not an incidental wrap. `white-space: pre-wrap` preserves those
newlines, and the exact [`LINE_HEIGHT`](Self::LINE_HEIGHT) makes a caret's `y`
readable as a line index.

[`scrolling_sheet`](Self::scrolling_sheet) adds the one thing
[`sheet`](Self::sheet) withholds: a fixed height and `overflow: auto`, which
turns the same control into a VERTICAL scroll container — content taller than
three lines overflows, and the caret has to drag the scroll after it.
