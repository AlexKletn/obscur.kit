Returns the vertical [`LineMetrics`] for a specific `font` at `font_size`,
for the mixed-font line-box **strut** (CSS 2.1 §10.8.1).

A line box's used height and baseline come from the *max* ascent and *max*
descent over the fonts actually on that line, so the wrapper must be able
to ask for each segment font's own metrics — not just the primary face's.
The **default** ignores `font` and delegates to
[`line_metrics`](Self::line_metrics), so a font-agnostic shaper (the
[`MockTextShaper`], or any stub) reports the *same* metrics for every font
and a line's strut therefore equals the primary strut — keeping headless
output byte-identical. A real adapter that can load more than one face
overrides this to return that face's own scaled vertical metrics, so a
taller fallback run grows its line box.
