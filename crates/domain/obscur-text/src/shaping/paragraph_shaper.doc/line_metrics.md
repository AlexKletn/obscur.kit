Returns the vertical [`LineMetrics`] for a line at `font_size`, delegating
to the inner [`TextShaper`].

A paragraph shares one line box height and baseline across all of its
fonts (spec §5.10), so the inner shaper's metrics — the idealized ratios
for the mock, real font metrics for an adapter — drive line stacking for
the whole paragraph.
