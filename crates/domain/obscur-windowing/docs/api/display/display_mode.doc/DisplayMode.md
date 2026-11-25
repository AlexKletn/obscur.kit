A single video mode a display can present: a physical resolution paired with
an optional refresh rate (spec §6.6).

The refresh rate is stored in millihertz as an integer so the value stays
exact and deterministic across platforms: fractional rates such as 59.94Hz
are represented precisely as `59_940` mHz rather than as a lossy `f32`.
