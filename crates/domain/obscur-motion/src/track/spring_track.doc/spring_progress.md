Computes normalized spring displacement in `[0, ~1]` at progress `t`.

The analytic solution of `m·x'' + c·x' + k·x = 0` gives the *remaining*
displacement `r(t)`; progress is `1 − r(t)`. Time is `T = t · 6` so the
spring mostly settles within `t ∈ [0, 1]`. `f(0)` is exactly `0`;
underdamped springs overshoot past `1` before settling.
