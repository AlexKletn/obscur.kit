A value-driven [`MotionDriver`]: a continuous external signal mapped onto
`[0, 1]` (spec §4.17 "value-driven motion; drag/cursor/gesture signals").

The driver maps a raw signal `value` — a drag distance, cursor coordinate or
gesture magnitude — from an input domain `[min, max]` onto normalized
progress via `(value - min) / (max - min)`, clamped to `[0, 1]`. The domain
endpoints do not have to be ordered: an **inverted** domain (`min > max`)
normalizes correctly, mapping `value == min` to progress `0` and
`value == max` to progress `1`, so a signal that runs high-to-low still
drives forward. A degenerate domain (`min == max`) has no width to normalize
against and yields progress `0`.
