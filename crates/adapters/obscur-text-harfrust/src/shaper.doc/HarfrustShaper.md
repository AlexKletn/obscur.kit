A real [`TextShaper`] that shapes text with the HarfRust engine (spec §5.10).

The shaper holds the font **bytes** it shapes with. The primary path is
[`HarfrustShaper::from_bytes`]: the host wires one font and every request is
shaped against it (the request's own [`FontFace`] source is ignored, since it
may be a metadata-only placeholder). Alternatively [`without_font`] builds a
font-less shaper that falls back to per-request
[`FontSource::Memory`](obscur_text::font::FontSource::Memory) bytes carried on
the request's face.

Shaping is infallible: an empty run, absent font, or bytes HarfRust cannot
load all yield an empty [`GlyphRun`] rather than a panic, and identical
requests always produce identical output.

# Line metrics

[`line_metrics`](TextShaper::line_metrics) and
[`line_metrics_for`](TextShaper::line_metrics_for) are asked for a *size* and
at most a [`FontId`] — never for a face — so a font-less shaper can only
answer from real tables if it was handed the registry's
[`FaceMetrics`] with [`with_faces`](Self::with_faces). Do wire it: without it
the glyphs are real but every line box is sized from the idealized 0.8/0.2
ratios. Each query that has to fall back that way is counted by
[`idealized_line_metrics`](Self::idealized_line_metrics), so a host or test
can see the degradation instead of silently shipping fake line boxes.

[`without_font`]: HarfrustShaper::without_font
