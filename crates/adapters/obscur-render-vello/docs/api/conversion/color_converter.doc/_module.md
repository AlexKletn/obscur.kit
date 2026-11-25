Converts an obscur-scene [`Color`] into a Vello/peniko color.

Pure, backend-local conversion (spec §2.7): the domain authors colors as
straight-alpha sRGB `f32` components; this maps them to peniko's
`AlphaColor<Srgb>` (spec §5.2 — geometry and paint order are preserved by the
adapter; only rasterization is delegated to Vello).
