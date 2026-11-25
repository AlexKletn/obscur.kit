The pen position shared by every command here.

Both backends place a glyph at `origin + glyph.x()`, so a shared origin with a
shared per-glyph offset puts the two drawings on bit-identical absolute
positions. What is then measured is compositing, not f32 rounding.
