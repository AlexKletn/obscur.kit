Samples the presentation value at normalized progress `t` (spec §4.17).

`t == 0.0` yields the start value and `t == 1.0` the end value.
Bounded-easing tracks clamp `t` into `[0, 1]`; spring tracks pass it
through so overshoot is preserved. Pure and deterministic: sampling the
same track at the same `t` always yields the same value.
