The honest [`SupportLevel`] at which this software backend reproduces each
[`SceneEffect`] (spec §5.2).

Every arm below states what the paint path in [`Renderer::render`] *does*
with that effect's parameters — not what a software rasterizer could in
principle do:

- [`Image`](SceneEffect::Image) is [`Native`](SupportLevel::Native): a
  `DrawImage` blits the real decoded pixels through `draw_pixmap`, honoring
  the tile, region, and both repeat axes. A genuine sampled bitmap, not an
  approximation.
- [`Shadow`](SceneEffect::Shadow) is [`Limited`](SupportLevel::Limited): the
  shadow is drawn with its color, offset, spread, radii, and inset flag — but
  its **blur radius is ignored**, so the edge is hard. A
  `filter: drop-shadow(...)` reaches the same path and is equally limited.
- [`Text`](SceneEffect::Text) is `Limited`: a run whose font handle was given
  to [`CpuRenderer::register_font`] rasterizes as real glyph outlines; a run
  with no registered font falls back to box-coverage cells.
- [`Clip`](SceneEffect::Clip) is [`Emulated`](SupportLevel::Emulated): a
  `PushClip` shape becomes a per-pixel coverage mask, honoring the active
  transform and the enclosing clip. (A shape clip carried on a `PushLayer`
  would be dropped, but the coordinator only puts one there on filter layers,
  which are `Unavailable` here anyway.)
- [`Blur`](SceneEffect::Blur), [`BackdropFilter`](SceneEffect::BackdropFilter),
  and [`ColorMatrix`](SceneEffect::ColorMatrix) are
  [`Unavailable`](SupportLevel::Unavailable): these are the GPU-class effects
  of the effect-split policy and this backend has **no pass** for any of
  them. There is no gaussian anywhere in it: a blurred `DrawShadow` draws a
  hard rect, a `filter: blur()` layer draws its subtree sharp, a
  `BackdropFilter` draws nothing at all, and a color-matrix layer draws
  unfiltered. Dropping the parameter is not emulation.
- [`Gradient`](SceneEffect::Gradient) is `Unavailable`: this is a single
  coarse flag over all gradient kinds, and it stays `Unavailable` because not
  every kind renders. Linear (premultiplied-sampled shader) and conic (wedge
  sweep) brushes DO rasterize for real, but a **radial** brush still degrades
  to a flat solid of its first stop — its stops and geometry are dropped. The
  flag reports the weakest case rather than over-claiming full support.
- [`Mask`](SceneEffect::Mask) is `Unavailable`: a mask clip's coverage is
  never applied (it inherits the enclosing clip instead), on this backend and
  on Vello alike.
- [`BlendMode`](SceneEffect::BlendMode) is `Unavailable`: a layer composites
  with `SrcOver` only; its blend mode is dropped.
