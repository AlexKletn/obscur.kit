An sRGB color with straight alpha, as authored (spec §5.7).

`Color` is deliberately just the component tuple `{ r, g, b, a }`. Its
interpretation — the [`ColorSpace`](super::ColorSpace) the components live in
and the [`AlphaMode`](super::AlphaMode) (straight vs. premultiplied) — is
tracked by the surrounding paint at each use site rather than stored here,
keeping `Color` a small, copyable value. By convention the components are
straight-alpha sRGB in the `0.0..=1.0` range; values outside that range are
permitted (e.g. for wide-gamut or HDR authoring) and are not clamped.
