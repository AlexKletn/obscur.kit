The one origin every command in a case shares.

Both sides of every comparison must hand Vello **bitwise identical** absolute
glyph positions, or the test would be measuring float rounding instead of the
fold. `draw_text` computes `origin + glyph.x()`, so sharing the origin and
varying only the run-relative offsets makes the two sides exact by construction.
