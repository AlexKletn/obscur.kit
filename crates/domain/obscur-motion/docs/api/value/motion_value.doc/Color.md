An sRGB color with straight alpha, `[r, g, b, a]`, each in `0.0..=1.0`.

Unlike the numeric variants this does **not** blend straight component-
wise: it interpolates with **premultiplied** alpha (css-color-4 §13.3), so
a transition from an opaque color to a transparent one keeps the visible
hue full-strength instead of muddying it toward the transparent endpoint's
channels. See [`interpolate_checked`](Self::interpolate_checked).
