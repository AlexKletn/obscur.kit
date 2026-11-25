Returns the natural line height in font units:
`ascent + |descent| + line_gap` (spec §5.18).

The descent's magnitude is used so a font's conventional negative descent
still adds to the line height rather than subtracting from it.
