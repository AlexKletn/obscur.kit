Draw a run of pre-shaped text. A LEAF command (no nesting effect). The
`origin` is the run's top-left in absolute scene coordinates; `run` carries
the positioned glyphs produced **once** by the `obscur-text` domain, so
every renderer consumes an identical run and only rasterization differs
(spec §5.10, §5.2). Renderers map each glyph to an outline and draw it at
its run-relative position — they never re-shape.
