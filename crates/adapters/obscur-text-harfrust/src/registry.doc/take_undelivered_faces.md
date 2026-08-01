Takes the discovered faces the renderer has not been given yet, as
`(id, bytes)`.

A face found mid-frame exists only in this registry; the renderer
rasterizes from its own id→bytes table and would draw nothing for a glyph
whose face it has never seen. A host drains this after each frame and
hands what it finds to the renderer — the same id→bytes hand-off
`InstalledFonts` performs at startup, continued for the faces that arrive
later.
