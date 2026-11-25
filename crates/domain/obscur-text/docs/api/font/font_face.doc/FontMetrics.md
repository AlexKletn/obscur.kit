A font's design metrics, in font design units (spec §5.18).

Distances are expressed in the font's own coordinate space, whose scale is
[`units_per_em`](FontMetrics::units_per_em); divide by it and multiply by a
pixel size to obtain pixels. Following font conventions, `descent` is
typically **negative** (below the baseline); it is stored exactly as given,
and [`line_height`](FontMetrics::line_height) takes its magnitude.
