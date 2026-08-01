The real design metrics of a set of registered faces, keyed by [`FontId`] and
shared cheaply by [`Arc`] (spec §5.10).

A [`TextShaper`](obscur_text::shaping::TextShaper) is asked for a font's
[`LineMetrics`] by **id** only —
[`line_metrics_for`](obscur_text::shaping::TextShaper::line_metrics_for)
receives a [`FontId`] and a size, never a face — so a shaper that does not
itself hold the bytes has no way to answer from real tables. Only the
[`FontRegistry`](crate::FontRegistry) that issued the ids knows which face
each id names; `FaceMetrics` is the table it hands over
([`FontRegistry::face_metrics`](crate::FontRegistry::face_metrics)) so
[`HarfrustShaper::with_faces`](crate::HarfrustShaper::with_faces) can resolve
an id to that face's own `hhea` metrics.

The table is a **snapshot** taken when [`face_metrics`] is called: faces
registered afterwards are absent, and an absent (or unreadable) face reports
[`None`], which is what makes the shaper's idealized fallback observable
rather than silent.

[`face_metrics`]: crate::FontRegistry::face_metrics
