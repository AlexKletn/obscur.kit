Rasterizes an **inset** `box-shadow` as an inner frame confined to the box —
the software backend's blur-free approximation of a CSS inner shadow (spec §5.2).

The shadow darkens the inside edges of the box, so it is drawn as a rounded-rect
*ring*: the border box `rect` (with `radii`) as the outer outline, and the box
deflated by `spread` and shifted by `offset` as the inner hole, filled with the
even-odd rule ([`Path::rounded_rect_ring`]). Because the ring lives entirely
inside the border box, the shadow is naturally confined to the box (an inset
shadow never bleeds outside it). With `spread > 0` and no blur this is a crisp
inner ring; an `offset` makes the frame thicker on the side opposite the shift —
the inner-shadow look. The `blur` radius is unavailable on this backend, so the
frame is hard-edged (documented approximation); it keeps the shadow color's own
alpha, since the ring geometry is exact and there is no softness to fake. A
collapsed inner rect (huge `spread`) fills the whole box; a degenerate box draws
nothing.
