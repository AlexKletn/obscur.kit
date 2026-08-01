The first `DrawText` run's advance width and right edge (`origin.x + Σadvance`).
The right edge is where the last glyph (and the caret trailing it) paints; the
width proves the run is the FULL value, not a box-width truncated prefix.
