A `from` → `to` motion shaped by a normalized spring curve (spec §4.17).

This is the native compile target of a spring `animation`: rather than a
fixed easing table, the progress is the analytic displacement of a damped
harmonic oscillator (`m·x'' + c·x' + k·x = 0`) mapped onto `t ∈ [0, 1]`. A
*full* velocity-carrying physical spring — one that accepts an initial
velocity and integrates real time to rest — is a driver-level concern; here
the spring is a pure, deterministic **normalized curve** with `f(0) == 0` and
`f(1) ≈ 1`.

Springs overshoot: for an underdamped configuration the progress can exceed
`1.0` (and dip back) before settling. That is intentional and desirable — the
underlying [`MotionValue`] interpolation does not clamp, so the sampled value
overshoots its target the way a physical spring would.
