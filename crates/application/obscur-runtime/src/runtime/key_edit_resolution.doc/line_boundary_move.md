Moves the caret to the start (`to_end = false`, Home) or end (`to_end =
true`, End) of its **visual line**, extending the selection when `Shift` is
held (spec §5.11 "Home/End are visual-line, not paragraph, operations").

The target byte is resolved against the SAME wrapped layout the caret paints
from ([`caret_visual_line_bound`]), so on soft-wrapped text Home/End stop at
the wrap boundary rather than at a `\n` or the buffer end. When the field has
no arranged text box to wrap against, this falls back to the buffer's logical
line-boundary motion — byte-identical for a single-line or hard-`\n`-only
field, which has no soft wrap. Always consumes the key.
