The computed value of `object-fit`: how a replaced element's content (e.g. an
`<img>`'s decoded pixels) is scaled to fit its content box.

The scene coordinator turns this into a concrete tile rect within the element's
box at paint time, reusing the same cover/contain/fit geometry as
`background-size`:

* [`ObjectFit::Fill`] — stretch to exactly fill the box, ignoring the intrinsic
  aspect ratio (the CSS replaced-element default).
* [`ObjectFit::Contain`] — scale to the largest size that fits inside the box,
  preserving the aspect ratio (letterboxed).
* [`ObjectFit::Cover`] — scale to the smallest size that fully covers the box,
  preserving the aspect ratio (overflow clipped to the box).
* [`ObjectFit::None`] — draw at the intrinsic pixel size, unscaled.
* [`ObjectFit::ScaleDown`] — the smaller of [`ObjectFit::None`] and
  [`ObjectFit::Contain`]: like `none`, but shrunk to fit when the intrinsic size
  is larger than the box.
