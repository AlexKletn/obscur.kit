The honest [`SupportLevel`] at which this GPU backend reproduces each
[`SceneEffect`] (spec §5.2).

- [`Gradient`](SceneEffect::Gradient), [`Clip`](SceneEffect::Clip), and
  [`Image`](SceneEffect::Image) are [`Native`](SupportLevel::Native): a
  gradient brush becomes a peniko gradient over the same geometry and
  stops, a clip becomes a bounded `push_layer`, and a `DrawImage` a real
  sampled `draw_image`.
- [`BlendMode`](SceneEffect::BlendMode) is [`Limited`](SupportLevel::Limited),
  because the variant names two inputs and this backend honours one. A
  layer's `blend_mode()` **field** becomes a peniko `Mix` natively. A
  [`LayerEffect::Blend`](obscur_scene::layer::LayerEffect::Blend) in the
  layer's *effects list* is **dropped**: `push_layer` reads the field, and
  the effects list is scanned only for `Blur` and `ColorMatrix`. Reporting
  `Native` claimed both. Nothing produces a `LayerEffect::Blend` today — it
  duplicates the field — so this is latent, not live.
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
  the layer to carry a bounded clip, and does not recurse. **A filter layer
  nested inside another filter layer is drawn unfiltered**, as is one with no
  clip bounds. A `DrawShadow`'s own blur radius has no such restriction.
- [`Text`](SceneEffect::Text) is `Limited`: glyphs draw against the font
  registered for the run's handle; a run whose font was never registered is
  **skipped entirely**.
- [`Mask`](SceneEffect::Mask) is [`Unavailable`](SupportLevel::Unavailable):
  a mask clip converts to the unbounded outline, so its coverage is never
  applied — the same as on the CPU backend (spec §5.2: clipping must match
  across backends).
