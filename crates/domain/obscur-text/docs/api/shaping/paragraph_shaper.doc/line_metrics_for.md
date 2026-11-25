Returns the vertical [`LineMetrics`] of a specific `font` at `font_size`,
for the mixed-font line-box **strut** (CSS 2.1 §10.8.1), delegating to the
inner [`TextShaper`].

Where [`line_metrics`](Self::line_metrics) gives the primary/default face's
metrics (the strut floor), this asks for one segment font's own metrics so
the wrapper can grow a line box to the *max* ascent/descent over the fonts
on that line. Under the font-agnostic mock every font reports the same
metrics (the inner shaper's default), so the strut equals the primary strut
and output stays byte-identical; a real multi-face backend returns
face-specific metrics.
