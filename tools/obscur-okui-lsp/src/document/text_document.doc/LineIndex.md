A precomputed table of line-start byte offsets, enabling O(log n) conversion
between absolute byte offsets and zero-based [`Position`]s.

Line breaks are `\n`; a `\r` is treated as an ordinary byte belonging to the
line it precedes (so `\r\n` documents still map sensibly, with the `\r` as
the last byte of the line). `line_starts[0]` is always `0`, and there is one
entry per line; the final line has no trailing entry.
