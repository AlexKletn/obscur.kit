Returns this shaper with `faces` — a
[`FontRegistry`](crate::FontRegistry)'s
[`face_metrics`](crate::FontRegistry::face_metrics) snapshot — as the
source of per-[`FontId`] line metrics (spec §5.10).

This is the only way a shaper can map the id in a
[`line_metrics_for`](TextShaper::line_metrics_for) query back to the face
that issued it, so a registered face's line height and baseline come from
its own `hhea` rather than the idealized ratios.
