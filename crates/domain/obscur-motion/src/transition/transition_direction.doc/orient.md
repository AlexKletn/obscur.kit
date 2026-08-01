Maps raw progress `t` to directional progress (spec §4.17 "reversal").

`t` is first clamped to `[0, 1]`. `Forward` returns `t` unchanged;
`Reverse` returns `1 - t`, so raw progress `0` sits at the target and
raw progress `1` sits at the origin. The result is the progress an
endpoint-parameterised curve (easing, interpolation) should be sampled
at.
