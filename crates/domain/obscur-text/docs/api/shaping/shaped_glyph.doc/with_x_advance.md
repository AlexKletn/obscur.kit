Returns this glyph with its horizontal pen advance replaced by `x_advance`,
leaving the glyph id, cluster, and placement offsets untouched.

Justification uses this to widen the trailing glyph of an inter-word gap so
the line fills the available width without disturbing glyph identity or the
cluster mapping caret hit-testing relies on.
