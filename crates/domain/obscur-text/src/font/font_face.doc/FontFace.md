A concrete, resolvable font face: a family and attributes paired with the
source of its bytes and its design metrics (spec §5.18).

A [`FontProvider`](crate::font::FontProvider) produces a `FontFace` when it
resolves a [`FontFamily`] and [`FontAttributes`] request. Because it embeds
`f32` [`FontMetrics`] it derives only `Debug`, `Clone`, and `PartialEq`
(floats preclude `Eq`).
