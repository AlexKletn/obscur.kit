The honest [`SupportLevel`] at which this GPU backend reproduces each
[`SceneEffect`] (spec §5.2).

- [`Gradient`](SceneEffect::Gradient), [`Clip`](SceneEffect::Clip), and
  [`Image`](SceneEffect::Image) are [`Native`](SupportLevel::Native): a
  gradient brush becomes a peniko gradient over the same geometry and
  stops, a clip becomes a bounded `push_layer`, and a `DrawImage` a real
  sampled `draw_image`.
- [`BlendMode`](SceneEffect::BlendMode) is [`Native`](SupportLevel::Native)
  in classic Vello. The scene layer resolves the backdrop blend mode and
  Vello applies it to the group.
- [`Shadow`](SceneEffect::Shadow) and
  [`BackdropFilter`](SceneEffect::BackdropFilter) are
  [`Emulated`](SupportLevel::Emulated): both are real gaussians — a shadow
  through `draw_blurred_rounded_rect`, a backdrop through the owned segmented
  wgpu compositing path — but assembled from primitives rather than a single
  native op.
- [`Blur`](SceneEffect::Blur) and [`ColorMatrix`](SceneEffect::ColorMatrix)
  are [`Limited`](SupportLevel::Limited): a `filter: blur()` / color-op layer
  runs through our owned wgpu passes (a real gaussian, a real per-pixel
  affine), but only when the layer is peeled off the main scene — which needs
  the layer to carry a bounded clip. Classic Vello draws a filter nested
  inside a clip or compositing layer without the filter effect and reports
  the fallback. A layer with no clip bounds or its own blend, opacity, or composition
  also uses the fallback when the filter cannot be composited in place.
  This restriction applies to filter layers, while shadows use a separate blur path.
- [`Text`](SceneEffect::Text) is `Limited`: glyphs draw against the font
  registered for the run's handle; a run whose font was never registered is
  **skipped entirely**.
- [`ImageMask`](SceneEffect::ImageMask) is `Limited`. Classic Vello and
  Lyon apply image coverage once to the composited group. Classic Vello
  applies the mask before the filter pass when both are on the same
  extracted layer. Hybrid routes frames with image masks to classic Vello
  and reports that fallback to the diagnostic sink.
- [`Mask`](SceneEffect::Mask) is [`Unavailable`](SupportLevel::Unavailable).
  A mask without a coverage source uses the enclosing clip.
