Maps a domain [`Color`] (straight-alpha sRGB, `f32`) onto a
[`tiny_skia::Color`], clamping each component to `0.0..=1.0`. Returns `None`
only if `tiny-skia` rejects the clamped components.
