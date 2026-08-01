Resolves the flat solid color a brush rasterizes to.

A [`Brush::Solid`] yields its color directly. A gradient brush degrades to its
first stop color (spec §5.2 rasterization approximation); a stop-less gradient
yields `None`, so the command is skipped. Linear and conic fills no longer route
through here (they rasterize for real, see `fill_linear` / `fill_conic`); this
remains the path for strokes and radial fills.
