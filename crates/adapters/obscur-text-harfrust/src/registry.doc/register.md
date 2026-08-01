Registers a font (`bytes`) under `family`/`attributes` and returns the
[`FontId`] assigned to it — the next index in registration order (`0` for
the first font, `1` for the second, …).

The bytes are parsed once to read the face's `units_per_em` and `hhea`
vertical metrics for the resolved [`FontFace`]. Bytes that do not parse as
a font are still registered (keeping id assignment deterministic) with
idealized default metrics; such a font simply never covers any run, and
[`face_metrics`](Self::face_metrics) reports it as having no real metrics.
