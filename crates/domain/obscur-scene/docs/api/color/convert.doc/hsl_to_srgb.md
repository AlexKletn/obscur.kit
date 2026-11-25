Converts `hsl()` to gamma-encoded sRGB (CSS Color 4 §7, "HSL to sRGB").

`hue` is in degrees (any real value; reduced mod 360), `sat` and `light` are
fractions in `0.0..=1.0`. HSL is defined directly on gamma-encoded sRGB, so
no transfer function is applied. Result is clamped to `[0, 1]`.
