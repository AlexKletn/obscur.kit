Resolves a `to <corner>` keyword pair to its [`Corner`] and nominal diagonal
angle (the aspect-ratio-aware angle is computed against the box at paint time;
the nominal angle is a square-box fallback). The two keywords must name one
vertical (`top` / `bottom`) and one horizontal (`left` / `right`) edge, in
either order.
