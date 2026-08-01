A face discovered mid-frame has to reach the renderer, which rasterizes from
its own id→bytes table: resolving a glyph the renderer cannot draw is the
same blank, one layer down.
