Builds the per-line highlight for the byte span `start..end` over a wrapped
`layout` (spec §5.11 editing engine).

A selection that spans several wrapped lines paints one rectangle per
covered line: the **leading** line runs from the start offset's pen x to
that line's trailing edge, whole **interior** lines span their full used
width, and the **trailing** line runs from its content start to the end
offset's pen x. The rectangles are in *paragraph-local* coordinates — x is
the line's alignment [`x_offset`](ParagraphLine::x_offset) plus the
within-line pen x, y is the line box's [`top`](ParagraphLine::top) — so the
caller translates them by the text box origin (and any horizontal scroll)
to place them on screen.

The within-line pen x is the sum of THAT line's spaced/tabbed glyph
advances whose source cluster falls before the offset — the same advances
the line's glyphs paint from — so a span confined to ONE line yields exactly
the single rectangle the caret-driven highlight produced before. A collapsed
span (`start == end`) and any zero-width per-line extent (e.g. a downstream
caret's sliver at a soft-wrap boundary, or an empty line) contribute no
rectangle. Determinism: the covered lines and their advances are a pure
function of the layout and the two offsets.
