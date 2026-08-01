Converts an obscur-scene [`Color`] into a [`peniko::Color`].

Each channel is clamped to `[0.0, 1.0]`; the alpha is straight (not
premultiplied), matching peniko's `AlphaColor<Srgb>`.
