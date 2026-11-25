Builds the even-odd rounded-rect ring painting a uniform border of `width`
around `bounds` (the border box) with authored corner `radii`.

The ring's outer outline is the border-box rounded rect; its inner outline is
the padding box — `bounds` inset by `width` on every side (clamped to a
non-negative size) — carrying the §4.2 inner radius. Filled even-odd, the inner
rounded rect becomes a hole so the border's inner corners are rounded.

CSS Backgrounds 3 §4.2: "The padding edge (inner border) radius is the outer
border radius minus the corresponding border thickness. In the case where this
results in a negative value, the inner radius is zero." *Corresponding* is the
load-bearing word, and it is per axis: a corner's horizontal radius is offset by
the thickness of the border it meets horizontally (left/right) and its vertical
radius by the one it meets vertically (top/bottom). This function is the
**uniform**-border path — its caller has already established that all four
border widths are equal (see the `ct == cr && ct == cb && ct == cl` guard), so
every corresponding thickness is the same `width` and both axes reduce by it.
That is [`CornerRadii::offset`] with a negative delta, which floors at zero for
§4.2's negative case and keeps a sharp corner sharp.
