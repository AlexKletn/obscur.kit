Collapses a shadow's eight radii into the one scalar Vello's blur can take.

**This is a Vello backend limitation, and it is lossy.** `vello 0.9`'s
`Scene::draw_blurred_rounded_rect(.., radius: f64, std_dev: f64)` — and its
`_in` variant — accept a *single* `f64` radius for the whole rect. There is no
per-corner form and no elliptical form (kurbo's `RoundedRectRadii`, which the
inset path's `to_rounded_rect` could take, is four **scalars** — per-corner but
still circular, so it does not rescue the elliptical case either). So a shadow
whose box has elliptical corners (`border-radius: 50%` on a non-square box) or
unequal corners cannot be reproduced exactly on Vello, and this function picks
the closest circle rather than a silent axis.

Two losses, in order:

1. **Per corner**, [`CornerRadius::circular_approximation`] takes the mean of the
   two axes — exact when the corner is already circular, unbiased when not.
2. **Across corners**, the max wins, which over-rounds a box whose corners
   differ. This is the pre-existing convention and is exact for the overwhelmingly
   common case of four equal corners.

Both losses vanish for a uniform circular `border-radius`, which is why the
normal path is exact. This divergence is confined to the **blurred shadow**
effect and does not touch clip or scene geometry, which spec §5.2 requires to
match across backends: those go through [`obscur_scene::path::Path`], which draws
true quarter ellipses on both backends. The CPU backend builds its shadow from
that same `Path` and so keeps the ellipse; Vello's blurred shadow rounds it. See
`docs/conformance-audit.md` finding 4-A.

[`CornerRadius::circular_approximation`]: obscur_geometry::CornerRadius::circular_approximation
