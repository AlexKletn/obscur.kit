The bases a relative length resolves against.

* `font_size` — the element's own computed `font-size`, in logical pixels;
  the base for `em`.
* `root_font_size` — the document root's computed `font-size`, in logical
  pixels; the base for `rem`.
* `viewport_width` / `viewport_height` — the viewport dimensions in logical
  pixels; the bases for `vw` / `vh` (and, combined, `vmin` / `vmax`).
* `ch_width` — the advance of the `0` (ZERO, U+0030) glyph in the element's
  own font, in logical pixels; the base for `ch`. It is [`None`] when that
  advance could not be measured, in which case [`ch_base`](Self::ch_base)
  substitutes the specified fallback — see there.
* `percent_basis` — the length a `%` resolves against, when it is known at
  resolution time. It is [`None`] when the basis is only known later (bare
  percentages are resolved by layout against the containing block, not here),
  in which case a `%` term cannot be folded and resolution returns [`None`].

Because every field is an `f32` (or an `Option<f32>`), this type derives
[`PartialEq`] but not [`Eq`]/[`Hash`].
