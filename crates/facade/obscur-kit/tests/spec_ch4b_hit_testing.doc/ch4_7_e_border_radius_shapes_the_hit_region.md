§4.7: "Hit testing accounts for:" / "- custom shapes;"

A 100x100 box with `border-radius: 50px` is a circle inscribed in its border
box, over a full-window background. (50, 50) hits the circle. (5, 5) is
inside the border box but `√(45² + 45²) ≈ 63.6` from the centre — beyond the
50px radius — so it misses the circle and hits the background beneath.
