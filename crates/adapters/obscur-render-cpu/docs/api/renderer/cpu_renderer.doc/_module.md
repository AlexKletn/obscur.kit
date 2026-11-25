A software/headless [`Renderer`] that rasterizes a scene to a CPU bitmap with
`tiny-skia` (spec §5.2 renderer abstraction, §2.7 "Software/headless
renderer").

[`CpuRenderer`] walks a [`SceneSnapshot`](obscur_scene::scene::SceneSnapshot)'s
display list — in paint order, maintaining the clip stack — and fills or
strokes each path into a [`tiny_skia::Pixmap`]. The last frame's pixels are
retained so a golden-image test can inspect them.

# Determinism boundary (spec §5.2)

This backend reproduces layout, paint order, clipping, and the consumed
Scene/Paint snapshots identically to every other backend; only rasterization,
anti-aliasing, and precision differ. Effects it cannot reproduce degrade to a
documented approximation rather than a panic — and, per §5.2, that degradation
is an **explicit diagnostic**: [`Renderer::supports_effect`] answers what this
backend will do with an effect before a host asks for it, and
[`Renderer::render`] emits [`RendererError::UnsupportedEffect`] into the
[`CpuRenderer::set_diagnostic_sink`] sink for every effect a scene demands that
the list below drops.

An effect this backend *does* draw, only approximately, is not in that set — it
is `Limited`, not `Unavailable` — so its restriction is emitted separately, and
only when the scene actually trips it: today that is a text run whose font is
not registered here. The other
approximations below are unconditional properties of the rasterizer, declared by
`supports_effect` rather than announced per frame. The approximations:

- **Gradient brushes**: linear, radial and conic fills use premultiplied
  Oklab samples. Radial strokes use the same shader as radial fills.
  Linear and conic strokes and gradient text use their first stop color.
  A stop-less gradient is skipped.
- **Layer compositing** ([`PushLayer`]/[`PopLayer`]) honors **group opacity**:
  a layer with opacity `< 1.0` (a CSS `opacity` group) rasterizes its whole
  subtree into a fresh offscreen [`tiny_skia::Pixmap`] and composites it back
  at that opacity, so overlapping opaque children blend as one group (not
  double-darkened per element) and nested layers multiply. A trivial layer
  (opacity `1.0`) stays a cheap passthrough — children draw directly, so the
  plain path is byte-identical. A layer's own **clip** *is* honored: it
  resolves through the same rule as a [`PushClip`] of the same region, so a
  layer clip and a clip command confine drawing identically and agree with the
  GPU backend (spec §5.2). The layer's **blend mode, transform, and effects**
  are still *not* honored here — a documented approximation; `filter` /
  `backdrop-filter` remain absent.

[`PushClip`]: obscur_scene::command::PaintCommand::PushClip
- **Mask clips** (as opposed to geometric shape clips) inherit the current
  effective clip, since coverage from separate source content is not
  rasterized here.
- **Box shadows** ([`DrawShadow`]) rasterize as a flat, offset rounded rect at
  reduced alpha (no gaussian blur; the `blur` radius is ignored) — a documented
  approximation of the soft shadow the GPU backend blurs. A CSS
  `filter: drop-shadow(...)` reaches the same [`DrawShadow`] path (the
  coordinator emits a drop shadow of the border box), so it rasterizes
  identically; the rest of `filter` / `backdrop-filter` (content blur,
  color-matrix) remain absent.
- **Images** ([`DrawImage`]) blit the real decoded RGBA8 pixels: the buffer is
  premultiplied (as `tiny-skia` stores) if the
  [`SceneImage`](obscur_scene::paint::SceneImage) is straight-alpha,
  wrapped into a source [`tiny_skia::Pixmap`], and drawn with `draw_pixmap` under
  a transform that scales the image's intrinsic size to each copy's tile rect.
  This is a genuine sampled bitmap, not an approximation, so both backends show
  the same picture. The tile is placed (`background-position`), sized
  (`background-size`), clipped to the paint region (the border box), and tiled
  along the `repeat_x` / `repeat_y` axes (`background-repeat`) with a bounded
  loop; a source sub-rectangle and nine-slice are not modeled yet.

[`DrawImage`]: obscur_scene::command::PaintCommand::DrawImage

- **Text runs** ([`DrawText`]) consume the same pre-shaped
  [`GlyphRun`](obscur_scene::text::GlyphRun) the GPU backend does — text is
  shaped once in `obscur-text`, never here — so both backends agree on glyph
  count and position (spec §5.2 determinism boundary). When the run's font has
  been registered with [`CpuRenderer::register_font`] (mirroring the Vello
  backend's registry), each glyph's real outline is extracted with `skrifa`,
  scaled to the run's font size, placed on its baseline, and filled — honest
  glyph shapes, deterministic for a given font. When no font is registered for
  the run's handle, the backend falls back to a solid coverage cell of the
  glyph's advance width and the run's font size (whitespace glyphs draw
  nothing) — a documented approximation of the mark so device-free callers
  still get a defined result.

[`DrawText`]: obscur_scene::command::PaintCommand::DrawText

[`DrawShadow`]: obscur_scene::command::PaintCommand::DrawShadow

[`PushLayer`]: obscur_scene::command::PaintCommand::PushLayer
[`PopLayer`]: obscur_scene::command::PaintCommand::PopLayer
