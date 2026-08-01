Converts CIE Lab (D50) to gamma-encoded sRGB (CSS Color 4 §8 + §10.4).

`l` is the lightness in `0.0..=100.0`; `a`/`b` are signed. Steps: Lab -> XYZ
(D50 white) -> linear sRGB via the CSS Color 4 combined Bradford-adapted
matrix (D50 XYZ straight to linear sRGB) -> encode -> clamp.
