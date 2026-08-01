The optional rounded-rect mask a [`composite_pass`](GaussianBlur::composite_pass)
clips its blurred quad to — the backdrop-filter panel shape.

[`Default`] is the **disabled** mask (`quad == [0, 0]` → coverage constant `1`),
so the `filter: blur` composite stays byte-identical. A constructed mask carries
the panel border box + per-corner radii in the composited quad's own pixel
space, plus the antialiased edge half-width `aa`.
