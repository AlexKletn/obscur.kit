A stateless converter from an obscur-scene [`Color`] to a [`peniko::Color`].

obscur-scene stores colors as straight-alpha sRGB `f32` components that are
deliberately *not* clamped (to allow wide-gamut / HDR authoring). peniko's
`AlphaColor<Srgb>` is likewise an `f32` sRGB color, so the conversion is a
direct component copy; components are clamped to `[0.0, 1.0]` here because the
Vello pipeline expects displayable sRGB values.
