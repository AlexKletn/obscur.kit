Resolves one computed `border-*-radius` corner (`id`) into its two radii
against the node's `bounds` (its border box).

A corner radius is the one place a `<length-percentage>` needs the box on BOTH
axes at once. CSS Backgrounds 3 §4.1: "Percentages for the horizontal radius
refer to the width of the border box, whereas percentages for the vertical
radius refer to the height of the border box." So one authored `50%` becomes
two different numbers — `(w/2, h/2)` — and `border-radius: 50%` is a circle
only on a square box; on a 2:1 box it is an ellipse. That is why this resolves
**here**, in paint, rather than in the resolver's absolutize pass: computed-value
time does not have the box (see [`border_width_of`]).

* `Px` (what every `em` / `rem` / `vw` / `vh` has already folded to) is the same
  radius on both axes — a circular corner.
* `Percent` resolves per axis against `bounds`. Without a `bounds` — a node the
  layout never arranged — a `%` has no basis and yields a square corner.
* Anything else (`auto`, an unmeasured `ch`) yields a square corner, as before.
