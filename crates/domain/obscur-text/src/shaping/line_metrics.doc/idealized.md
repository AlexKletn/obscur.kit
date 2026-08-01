The idealized metrics for `font_size`: `ascent = 0.8·size`,
`descent = 0.2·size`, `line_gap = 0` — so
[`line_height`](Self::line_height) equals `font_size` exactly.

This is the deterministic fallback used when no real font is available
(the [`TextShaper`](crate::shaping::TextShaper) trait default and any
no-font adapter path), which is what keeps headless output reproducible.
