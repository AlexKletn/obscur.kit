One `DrawText` command: the given glyph ids at the given run-relative offsets,
white at the given alpha, at `ORIGIN` and `SIZE`.

Both the "two commands" and the "one command, two glyphs" cases are built from
this, so the only thing that differs between them is how many commands the
glyphs are spread across.
