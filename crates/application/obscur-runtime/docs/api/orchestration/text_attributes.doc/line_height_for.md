Resolves a text node's authored `line-height` into a logical-pixel line box
height from its computed `line-height`, the node's own `font_size`, and the
document `root_font_size` (§4.14, §5.10).

Reads [`PropertyId::LineHeight`], returning:
* `Some(multiple * font_size)` for a unitless [`PropertyValue::Number`]
  (a multiple of the font size, the common `line-height: 1.5` form);
* `Some(px)` for a [`PropertyValue::Length`], with relative units resolved
  against the node's font context (CSS Values 4 §5.1.1): `px` verbatim
  (`line-height: 24px`), `em` × `font_size`, `rem` × `root_font_size`, and a
  percentage `pct% × font_size` (CSS 2.1 §10.8.1 / CSS Text 3 §8: a `%`
  line-height resolves against the element's own font size);
* [`None`] for the initial `normal` keyword, an absent property, or any unit
  this metric does not resolve here (viewport units) — the caller then falls
  back to the font's natural (metric) line height, keeping the `normal`
  default byte-for-byte identical to pre-`line-height` behaviour.

The absolute-`px` path is unchanged (byte-identical to the previous
`Length::as_px` reading); only the previously-dropped `em` / `rem` / `%`
units now resolve instead of silently falling back.

The same value drives both the layout measure/arrange pass and the scene
compose pass, so the sized box height and the painted line spacing agree.
