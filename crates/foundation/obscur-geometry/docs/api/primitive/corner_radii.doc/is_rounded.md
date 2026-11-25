Whether this corner actually rounds.

Per CSS Backgrounds 3 §4.1, "if either length is zero, the corner is
square, not rounded" — a quarter ellipse with a zero axis has no area. So
this is a conjunction, not a disjunction: `50%` on a zero-height box gives
`y == 0` and a square corner.
