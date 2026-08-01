Offsets every corner by `delta` on **each axis independently**, keeping a
sharp (zero) corner sharp and flooring at `0`.

This is the neutral geometric primitive shared by the two CSS rules that
grow or shrink a set of radii by a distance. It is deliberately *not* named
for either of them, because they do not agree:

* **CSS Backgrounds 3 §4.2 (corner shaping)** — "The padding edge (inner
  border) radius is the outer border radius minus the corresponding border
  thickness. In the case where this results in a negative value, the inner
  radius is zero." That is exactly this method, and the *whole* of it: §4.2
  prescribes a plain subtraction with no easing.
* **CSS Backgrounds 3 §6.1.1 (`box-shadow` spread)** — the shadow's radii are
  the box radii ±spread, "applied independently to the radii in each
  dimension", which is this method **plus** a correction this method does not
  implement: when a radius is *smaller* than the spread, §6.1.1 first scales
  the spread by `1 + (r−1)³` (where `r` = radius ÷ spread) so a nearly-sharp
  corner stays sharp rather than ballooning. A 10px radius with 20px spread
  should give 27.5px; this method gives 30px.

**Do not add the §6.1.1 easing here** — it would silently corrupt §4.2, which
forbids it, and the two other callers that legitimately want a raw distance:
the dashed-border centerline, and `outline`'s concentric growth (`outline` is
CSS UI 3 §4 — it is not a CSS Backgrounds property at all — and neither it
nor `outline-offset` (§4.5) defines any easing of its own). §6.1.1 has its own
method: [`CornerRadii::spread_offset`].

Sharp-stays-sharp is not a shortcut: it is the `r == 0` endpoint both rules
agree on (§6.1.1's easing factor is `1 + (0−1)³ = 0` there), and it follows
from §4.1 — a corner with a zero axis "is square, not rounded", so there is no
curve for a distance to grow.
