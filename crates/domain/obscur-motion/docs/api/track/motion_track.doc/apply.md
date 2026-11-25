Maps normalized progress `t` through this easing curve (spec §4.17).

`t` is clamped to `[0, 1]` first, so every variant satisfies
`apply(0) == 0` and `apply(1) == 1` and stays within `[0, 1]` in between.
