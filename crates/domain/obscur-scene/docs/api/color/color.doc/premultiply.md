Returns the premultiplied form of this color: `(r*a, g*a, b*a, a)`.

This is a numeric transform only; the result still lives in the same
color channels and color space — it merely folds alpha into the color
channels, as required by premultiplied-alpha compositing (spec §5.7).
The [`AlphaMode`](super::AlphaMode) of the surrounding paint should be
updated to [`Premultiplied`](super::AlphaMode::Premultiplied) at the use
site.
