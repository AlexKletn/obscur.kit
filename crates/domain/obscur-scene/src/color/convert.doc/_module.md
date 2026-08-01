Canonical CSS Color 4 color-space conversions to sRGB.

Pure, deterministic math that lowers the CSS Color 4 color functions
(`hsl()`, `oklab()`, `oklch()`, `lab()`, `lch()`) down to gamma-encoded sRGB
channels in `0.0..=1.0`. Each public function returns clamped sRGB so that a
stylesheet's authored color always resolves to a paintable value.

The formulas are re-derived from the CSS Color 4 specification
(<https://www.w3.org/TR/css-color-4/>); nothing here is copied from an
existing browser engine. Section references are cited inline.

Gamut handling is a **simple per-channel clamp to `[0, 1]` after
conversion** (see [`clamp_srgb`]). Full CSS gamut mapping (the CSS Color 4
§13 binary-search chroma reduction in Oklch) is a deliberate later
refinement and is not implemented here.
