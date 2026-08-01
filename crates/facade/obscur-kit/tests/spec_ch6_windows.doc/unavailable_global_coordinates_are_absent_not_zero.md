§6.6 Displays: "On Wayland, unavailable global coordinates are represented
honestly."

A compositor that hides global coordinates yields an explicitly absent
origin — not `(0, 0)` — while scale and size stay present, and a window's
screen position is likewise absent. A backend that *does* expose coordinates
reports them present.
