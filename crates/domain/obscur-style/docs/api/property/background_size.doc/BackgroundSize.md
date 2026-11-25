The computed value of `background-size`: how one copy of a
`background-image` is scaled before it is placed and (optionally) tiled.

The scene coordinator resolves this against the painted box and the image's
intrinsic pixel size to a concrete tile width/height at paint time:

* [`BackgroundSize::Auto`] — draw the image at its intrinsic pixel size (the
  CSS initial). When both axes are `auto` the intrinsic size is used
  directly.
* [`BackgroundSize::Cover`] — scale the image (preserving its aspect ratio)
  to the smallest size that fully covers the box; overflow is clipped by the
  background paint region.
* [`BackgroundSize::Contain`] — scale the image (preserving its aspect ratio)
  to the largest size that fits entirely inside the box.
* [`BackgroundSize::Explicit`] — an explicit per-axis size, each component a
  [`Length`] (`px` / `%` of the box extent) or [`Length::Auto`]. A single
  `auto` axis is derived from the other axis to preserve the intrinsic aspect
  ratio; both `auto` collapse to the intrinsic size (like
  [`BackgroundSize::Auto`]).

Because [`Length`] carries `f32` data this derives [`PartialEq`] but not
[`Eq`]/[`Hash`].
