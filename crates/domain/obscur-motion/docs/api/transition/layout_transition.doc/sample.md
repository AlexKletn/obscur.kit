Samples the interpolated layout box at raw progress `t` (spec §4.17).

`t` is oriented by the [`direction`](Self::direction) (clamped to
`[0, 1]`), shaped by the [`easing`](Self::easing), then used to
interpolate from [`from_bounds`](Self::from_bounds) toward
[`to_bounds`](Self::to_bounds). Pure and deterministic.
