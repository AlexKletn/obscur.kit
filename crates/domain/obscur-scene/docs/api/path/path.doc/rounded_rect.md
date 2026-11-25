Builds a closed rounded-rectangle path.

Each corner is a **quarter ellipse** (CSS Backgrounds 3 §4.1) drawn with a
single cubic Bézier whose control points use the `KAPPA` constant — a
close but not exact approximation, which is standard for vector rounded
rectangles. The corner's horizontal radius governs the control points'
horizontal reach and its vertical radius the vertical one, so a
`border-radius: 50%` on a 2:1 box draws the ellipse the spec asks for
rather than a circle.

`radii` are floored at zero and put through the §4.5 overlapping-curves
correction ([`CornerRadii::normalized`]) for `rect`, so adjacent corners
can never overlap however the caller built them. That correction is
idempotent for radii that already fit, so a caller that has already
normalized (the scene coordinator does, once, for the whole node) pays
nothing and gets the same path.
