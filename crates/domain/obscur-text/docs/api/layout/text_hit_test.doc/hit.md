Resolves `point` to a caret position within `layout` (spec §5.10 caret).

The line is chosen as the one whose vertical box `[top, top + height)`
contains `point.y`, clamped to the first line above the paragraph and the
last line below it. Within the line the glyph advances are accumulated from
the line's `x_offset`; the glyph whose horizontal span contains `point.x`
gives the [`byte_offset`](Self::byte_offset) (its cluster) and the
[`is_trailing`](Self::is_trailing) flag (whether `point.x` is past the
glyph's midpoint). A point left of the first glyph clamps to the line's
start offset (not trailing); a point past the last glyph clamps to the
line's end offset (not trailing). An empty layout resolves to the origin
caret.
