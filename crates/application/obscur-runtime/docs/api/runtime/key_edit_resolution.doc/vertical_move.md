Moves the caret **vertically** between the wrapped lines of a multiline
field on ArrowUp (`down = false`) / ArrowDown (`down = true`), extending the
selection when `Shift` is held — standard caret vertical motion (spec §5.11).

A **single-line** field is never consumed ([`KeyEdit::Declined`]), so its
ArrowUp/ArrowDown fall through to the shortcut/activation path
byte-identically. In a multiline field the target byte is resolved against
the SAME wrapped layout the caret paints from ([`caret_line_move`]) at
roughly the caret's current x; when there is no line in that direction the
caret clamps to the text start (Up on the first line) or end (Down on the
last line).
