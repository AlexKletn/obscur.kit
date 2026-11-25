Lowers any `<color>` function to a [`StyleColor`], resolving each CSS Color 4
color space to gamma-encoded sRGB (spec CSS Color 4).

Handles `rgb()`/`rgba()`, `hsl()`/`hsla()` (§7), `oklab()` (§9.2),
`oklch()` (§9.3), `lab()` (§8.1) and `lch()` (§8.3). The non-sRGB spaces are
converted by the pure math in [`obscur_scene::color::convert`] and then
per-channel clamped into gamut (full CSS gamut mapping is a later
refinement). Any other function name yields [`None`].
