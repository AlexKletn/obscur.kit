Blends `self` toward `other` by the normalized progress `t`.

`t` is the progress in `[0, 1]`: `t == 0.0` yields `self`, `t == 1.0`
yields `other`, and intermediate values interpolate component-wise.
Implementations MUST NOT clamp `t` — easing curves, springs and
overshoot legitimately pass values outside `[0, 1]`, and the result is
expected to extrapolate linearly past the endpoints rather than
saturate.
