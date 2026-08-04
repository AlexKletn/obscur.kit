Draws the bar along an authored path instead of a built-in shape.

The path is SVG data — `"M 0 0 A 40 40 0 0 1 80 0"` — in the canvas's own
coordinates, where `(0, 0)` is its top-left. A string rather than geometry
because a `ViewValue` carries text and not paths, which is also what makes a
shape authorable from markup rather than from Rust alone.

Unparseable data falls back to the built-in shape: a bar that vanished because
of a typo in its outline would be a worse answer than a plain one.
