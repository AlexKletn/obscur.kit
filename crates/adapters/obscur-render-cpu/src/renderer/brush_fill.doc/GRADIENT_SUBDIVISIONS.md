The number of sub-samples inserted across each authored inter-stop span when a
linear gradient is handed to tiny-skia. tiny-skia lerps between adjacent stops in
straight-alpha space, so a span is pre-expanded into this many segments whose
endpoints already sit on the premultiplied curve; between them the residual
straight/premultiplied divergence is sub-perceptual. Even, so a probe at a span's
exact midpoint lands on an inserted sample.
