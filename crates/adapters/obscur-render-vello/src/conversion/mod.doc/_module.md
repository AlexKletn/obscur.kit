Pure obscur-scene → Vello/peniko/kurbo converters (spec §2.7).

All Scene/Paint → Vello conversions are local to this crate (spec §2.7) and
side-effect free. Each converter translates one part of the backend-agnostic
scene model into its Vello/peniko/kurbo equivalent while preserving geometry
and paint order (spec §5.2 determinism boundary); only rasterization is left
to Vello.

- [`ColorConverter`] — [`Color`](obscur_scene::color::Color) → peniko color.
- [`PathConverter`] — [`Path`](obscur_scene::path::Path) → kurbo `BezPath`.
- [`BrushConverter`] — [`Brush`](obscur_scene::paint::Brush)/`Stroke` →
  peniko `Brush`/kurbo `Stroke`.
- [`ClipConverter`] — [`Clip`](obscur_scene::clip::Clip) → kurbo clip shape.
- [`SceneConverter`] — [`DisplayList`](obscur_scene::command::DisplayList) →
  `vello::Scene`, driving the above in paint order.
- `shadow_painter` / `image_painter` — the two commands whose translation is a
  derivation rather than a mapping: a `DrawShadow`'s blurred rounded rect
  (css-backgrounds-3 §6.1) and a `DrawImage`'s tiled, clipped background layer
  (CSS Backgrounds §3).
- [`SceneSegment`] — one piece of a display list split at a `backdrop-filter`
  boundary, which the renderer composites in order.
