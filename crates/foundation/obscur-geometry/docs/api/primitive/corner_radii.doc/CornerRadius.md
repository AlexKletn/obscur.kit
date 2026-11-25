One corner's two radii, in logical pixels.

A CSS corner is a **quarter ellipse**, not a quarter circle: CSS Backgrounds 3
§4.1 defines the two `<length-percentage>` values of `border-*-radius` as "the
radii of a quarter ellipse that defines the shape of the corner", and resolves
them against different bases — "Percentages for the horizontal radius refer to
the width of the border box, whereas percentages for the vertical radius refer
to the height of the border box". So `border-radius: 50%` on a 200×100 box is
an ellipse (`x = 100`, `y = 50`), and only a square box makes it a circle. A
single scalar per corner cannot express that, which is why this type is a pair.

`x` runs along the horizontal edges, `y` along the vertical ones.
