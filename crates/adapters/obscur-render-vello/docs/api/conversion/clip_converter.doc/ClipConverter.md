A converter from an obscur-scene [`Clip`] to a kurbo [`BezPath`] clip shape.

It owns a [`PathConverter`] so every clip outline is built through the same
path conversion.

# Shape mapping

- [`ClipShape::Rect`] → the rectangle's four-line closed outline
  (via [`Path::rect`]).
- [`ClipShape::RoundedRect`] → the rounded-rectangle outline
  (via [`Path::rounded_rect`]), matching the domain's own rounding.
- [`ClipShape::Path`] → the path converted directly.
- [`Clip::Mask`] → the [`unbounded`](Self::unbounded) outline, which confines
  nothing. A mask carries only *how* to read its coverage, never the source
  the coverage comes from ([`Mask`](obscur_scene::clip::Mask)), so a mask clip
  in the command stream has no known extent — [`Clip::bounds`] reports `None`
  for exactly this reason. An empty `BezPath` would be the opposite claim
  (extent known, and empty), and Vello would clip the content away entirely;
  inheriting the enclosing clip asserts nothing the domain has not stated, and
  matches the CPU backend (spec §5.2 requires clipping to match across
  backends).

  This is the documented §5.8 fallback, **not** mask support: coverage is not
  applied, so a masked region paints as if unmasked on both backends. Neither
  backend rasterizes mask coverage today, and neither raises the §5.8
  diagnostic that an explicit fallback is supposed to carry.
