The color space a [`Color`](super::Color)'s components are expressed in
(spec §5.7).

Colors are authored and exchanged in `Srgb` (the external semantics), while
compositing, blending, blur and filters operate in `LinearSrgb` (the linear
working space). `DisplayP3` describes wide-gamut content; on displays that
cannot reproduce it, consumers fall back to a safe sRGB approximation.

This enum intentionally lists only the spaces needed by the current model.
HDR and additional spaces (e.g. Rec. 2020, `ACEScg`) are deferred to a later
step and will be added as new variants.
