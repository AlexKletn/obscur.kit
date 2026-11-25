Returns the vertical [`LineMetrics`] for a line at `font_size`, in logical
pixels (spec §5.10).

The layout pass uses [`line_height`](LineMetrics::line_height) to size a
text box and the scene pass uses [`baseline`](LineMetrics::baseline) to
place the glyph baseline within it. The **default** returns the idealized
ratios ([`LineMetrics::idealized`]) — `ascent = 0.8·size`,
`descent = 0.2·size`, `line_gap = 0`, so `line_height == font_size` — which
keeps the deterministic [`MockTextShaper`](super::MockTextShaper) and any stub shaper working
without real font tables. A real adapter overrides this to read the loaded
font's own vertical metrics.
