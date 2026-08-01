Samples the eased blend from `from` toward `to` at progress `t`.

The easing curve clamps `t` to `[0, 1]`, so `t <= 0` yields `from` and
`t >= 1` yields `to`. On a variant mismatch between the endpoints the
blend is undefined and this falls back to `to`.
