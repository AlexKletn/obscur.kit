The unit direction of a `to <corner>` gradient line over a `w × h` box, in
screen space (y down) (spec §3.1).

The gradient line points into the corner's quadrant and is perpendicular to the
diagonal joining the two neighbouring corners; that diagonal is `(w, h)`, so the
perpendicular pointing right is `(h, −w)` and each corner picks the signs that
aim into its quadrant (`+x` = right, `−y` = up). On a degenerate (zero-area)
box the direction falls back to straight down.
