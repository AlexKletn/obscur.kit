A converter from an obscur-scene [`DisplayList`] to a [`vello::Scene`].

It owns the leaf converters ([`PathConverter`], [`BrushConverter`],
[`ClipConverter`]) and drives them while replaying the command stream.

# Command mapping

- [`PaintCommand::DrawPath`] → [`Scene::fill`] (for a
  [`Paint::Fill`], using the path's own fill rule) or [`Scene::stroke`] (for a
  [`Paint::Stroke`]), each with the identity transform.
- [`PaintCommand::DrawImage`] → [`Scene::draw_image`]: the decoded RGBA8 pixels
  become a [`peniko::ImageData`] (zero-copy from the image's `SharedPixels`,
  whichever shape the bytes arrived in, tagged
  with its own alpha encoding) drawn at its intrinsic size, with a transform
  that scales each copy to its tile rect. The tile is placed / sized
  (`background-position` / `-size`), clipped to the paint region (border box)
  with a pushed clip layer, and tiled along the `repeat_x` / `repeat_y` axes
  (`background-repeat`) with a bounded per-copy loop.
- [`PaintCommand::PushClip`] → [`Scene::push_layer`] with a neutral blend and
  the clip's shape, which Vello treats as a clip region.
- [`PaintCommand::PushLayer`] → [`Scene::push_layer`] with the layer's blend,
  composition, and opacity. The layer's clip shape is used when present;
  otherwise a large finite rectangle stands in for an unbounded layer.
- [`PaintCommand::PopClip`] / [`PaintCommand::PopLayer`] → [`Scene::pop_layer`],
  guarded by a depth counter so an unbalanced list never triggers a stray pop.

# Layer approximations (spec §5.2)

A [`SceneLayer`] carries a transform, opacity, blend mode, Porter-Duff
operator, and effects. This converter maps opacity, blend mode, and operator
(see [`BlendMode`] / [`CompositionMode`] mapping) and clips to the layer's
shape. The layer transform and additional effects (e.g. blur) are **not**
applied here; they are the renderer sub-step's concern. Paint order and
nesting — the properties the determinism boundary requires — are preserved.

# Text

[`PaintCommand::DrawText`] carries a pre-shaped
[`GlyphRun`](obscur_scene::text::GlyphRun): text is shaped **once**, in the
`obscur-text` domain, so this adapter never re-shapes — it consumes the run's
glyph positions verbatim (the same run the software backend consumes, spec
§5.2). Rasterizing to *real* outlines still needs host fonts, supplied via
[`Self::register_font`] / [`Self::set_font`] (fonts are a host resource,
mirroring host-provides-device); with no font registered the converter
degrades gracefully and skips text runs. Each glyph's
[`glyph_id`](obscur_scene::text::PositionedGlyph::glyph_id) is already a
**font glyph index** (the domain shaped against the run's resolved font), so it
is used **directly** — no Unicode charmap remap — and drawn at the
domain-supplied position; Vello's glyph pipeline rasterizes it (spec §5.2).

## Multi-font runs

A [`GlyphRun`] carries a [`font`](GlyphRun::font) handle (`0` = primary): the
runtime emits one `DrawText` per font-segment, each tagged with its resolved
face's id. The converter keeps a registry keyed by that `u32` handle
([`Self::register_font`]) and draws each run against the matching font, so a
mixed-script line (e.g. Latin + Cyrillic resolved to two different faces)
rasterizes each segment from its covering font. An unknown handle falls back
to id `0`; with neither present the run is skipped (no panic).
