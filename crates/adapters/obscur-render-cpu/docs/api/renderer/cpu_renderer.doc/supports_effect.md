The honest [`SupportLevel`] at which this software backend reproduces each
[`SceneEffect`] (spec §5.2).

Every arm below states what the paint path in [`Renderer::render`] *does*
with that effect's parameters — not what a software rasterizer could in
principle do:

- [`Image`](SceneEffect::Image) is [`Native`](SupportLevel::Native): a
  `DrawImage` blits the real decoded pixels through `draw_pixmap`, honoring
  the tile, region, and both repeat axes. A genuine sampled bitmap, not an
  approximation.
- [`Shadow`](SceneEffect::Shadow) is [`Limited`](SupportLevel::Limited).
  Outset and inset shadows honor color, offset, spread, radii and Gaussian
  blur, with sigma capped at 16 device pixels.
- [`Text`](SceneEffect::Text) is `Limited`: a run whose font handle was given
  to [`CpuRenderer::register_font`] rasterizes as real glyph outlines; a run
  with no registered font falls back to box-coverage cells.
- [`Clip`](SceneEffect::Clip) is [`Emulated`](SupportLevel::Emulated): a
  `PushClip` shape becomes a per-pixel coverage mask, honoring the active
  transform and the enclosing clip. A shape clip on a layer also clips the
  layer contents.
- [`Blur`](SceneEffect::Blur) is `Limited`. Layers use a separable Gaussian
  in linear light over the painted extent plus the kernel margin. Sigma is
  capped at 16 device pixels. Exceeding the cap produces a diagnostic.
- [`BackdropFilter`](SceneEffect::BackdropFilter) is `Limited`. Backdrop blur
  samples the active target and replaces it through the rounded panel and
  enclosing clip. Sigma is capped at 16 device pixels.
- [`ColorMatrix`](SceneEffect::ColorMatrix) is `Limited`. The first matrix
  filters the layer in straight RGBA before compositing. Fully transparent
  pixels stay transparent, so matrices that create alpha from a transparent
  input are limited to the layer's painted coverage.
- [`Gradient`](SceneEffect::Gradient) is `Limited`. Linear, radial and
  conic fills and radial strokes render as gradients. Linear and conic strokes
  and gradient text use their first stop color.
- [`ImageMask`](SceneEffect::ImageMask) is `Emulated`. Alpha or linear
  luminance coverage is applied once to the composited group.
- [`Mask`](SceneEffect::Mask) is `Unavailable`. A declaration without a
  coverage source inherits the enclosing clip.
- [`BlendMode`](SceneEffect::BlendMode) is `Limited`. With source-over, the
  group uses the layer's effective blend mode. Source, destination-over, and
  clear use their composition operator. If a non-normal blend is also
  requested, the renderer reports that it omits the blend.
