Resolves a pointer at text-box-local `(local_x, local_y)` (already
scroll-adjusted; measured from the text node's box origin) to a byte offset
in `content`, picking the wrapped line by `local_y` then the byte by
`local_x` on that line — the 2D inverse of the line-aware caret paint
(spec §5.11). The line is the first whose box contains `local_y` (else the
last); within it the byte is the nearest cluster boundary to `local_x`
(`caret_offset_at_x`'s midpoint walk), clamped to the line's byte span so a
click never crosses onto an adjacent line, and kept before a trailing `\n`.
A single-line field → line 0 and the identical `caret_offset_at_x` byte
(byte-identical).
