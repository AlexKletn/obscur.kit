The §5-D regression, stated on its own so a failure names the bug.

Vello resolved a mask clip to an empty `BezPath`, which clips *everything*
away, while the CPU backend inherited the enclosing clip and painted. Same
scene, opposite pixels.
