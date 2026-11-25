How many pixels sit on opposite sides of half coverage in the two rasters.

The cross-backend comparison that survives antialiasing. The two rasterizers
weight edge pixels differently and are allowed to — the same allowance
`clip_cross_backend.rs` makes — so nothing here compares alphas pixel for pixel.
What must agree is where the ink is.
