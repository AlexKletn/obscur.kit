Resolves the byte a **vertical** caret move (ArrowUp/ArrowDown) lands on
from `index`, moving to the line above (`down = false`) or below
(`down = true`) at roughly the same pen x — standard caret vertical motion
(spec §5.11). Returns [`None`] when there is no line in that direction (the
caret is already on the first line moving up, or the last line moving down),
so the caller can clamp to the text start/end. A single-line field has one
line, so it always returns [`None`].
