The vertical metrics of a single line of text at a specific font size, in
**logical pixels** (spec §5.10).

Where [`FontMetrics`](crate::font::FontMetrics) carries a font's *design*
metrics in font units, `LineMetrics` is the same information already scaled to
a concrete `font_size`, so the runtime can size a text box and place its
baseline without knowing the font's units-per-em. A
[`TextShaper`](crate::shaping::TextShaper) produces it via
[`line_metrics`](crate::shaping::TextShaper::line_metrics).

Following the layout convention (and unlike raw font tables), [`descent`] is
stored as a **positive magnitude** below the baseline, so
[`line_height`](Self::line_height) is a plain sum.

[`descent`]: Self::descent
