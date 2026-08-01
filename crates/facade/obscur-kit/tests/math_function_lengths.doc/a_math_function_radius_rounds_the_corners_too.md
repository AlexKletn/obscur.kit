The case: `min()` picks 12px here (3vmin of a 400×300 viewport is 9px — so
the *smaller* term is the vmin one, and the corner is 9px). Either way it is
a curve, and before the fix it was four straight lines.
