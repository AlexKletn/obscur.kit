Offsets every corner by `delta` for CSS Backgrounds 3 **§6.1.1**
(`box-shadow` spread), easing a corner whose radius is smaller than the
distance it grows by.

`delta` is the signed distance the *radii* grow by, which is **not** the
authored `spread` for both shadow kinds: §6.1.1 expands an outer shadow's
perimeter and contracts an inner one's ("expanded outward (for outer
box-shadows) or contracted inward (for inner box-shadows)"), so a drop
shadow passes `+spread` and an inset shadow passes `−spread`. Growth is
therefore `delta > 0` whichever kind is asking.

Against [`CornerRadii::offset`] — the neutral §4.2 primitive — this adds
exactly one rule, and nothing else. §6.1.1:

> However, in order to create a sharper corner when the border radius is
> small (and thus ensure continuity between round and sharp corners), when
> the border radius is less than the spread distance (or in the case of an
> inner shadow, less than the absolute value of a negative spread
> distance), the spread distance is first multiplied by the proportion
> 1 + (r-1)³, where r is the ratio of the border radius to the spread
> distance, in calculating the corner radii of the spread shadow shape. For
> example, if the border radius is 10px and the spread distance is 20px
> (r = .5), the corner radius of the shadow shape will be
> 10px + 20px × (1 + (.5 - 1)³) = 27.5px rather than 30px. This adjustment
> is applied independently to the radii in each dimension.

**Only growth is eased.** Both cases the spec names — an outer shadow whose
radius is under a *positive* spread, and an inner shadow whose radius is
under the absolute value of a *negative* one — are cases where the radius
grows; §8.3's change log records that narrowing as its own entry ("Spread
radius adjustment is only applied to shadows and margins where the radius of
curvature grows, rather than shrinks", Issue 1900). A shrinking corner
(`delta < 0`) therefore takes the plain floored subtraction — the same
arithmetic [`CornerRadii::offset`] does.

"Applied independently to the radii in each dimension" is why `r` is taken
per axis rather than per corner: an elliptical corner can sit below the
spread on one axis and above it on the other, and each axis eases alone.

The two branches meet continuously: at `radius == delta`, `r == 1` and the
factor is `1 + 0³ == 1`, which is the raw addition. At the other endpoint a
sharp corner stays sharp — `r == 0` gives `1 + (−1)³ == 0` — which is also
what the shared `is_rounded` gate enforces. Those two agreeing endpoints are
why a test that only probes them cannot tell this method from `offset`.
