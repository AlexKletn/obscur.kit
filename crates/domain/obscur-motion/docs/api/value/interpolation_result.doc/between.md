Blends `from` toward `to` by `t`, classifying the outcome (spec §4.17).

Delegates to [`MotionValue::interpolate_checked`]: a `Some` result
becomes [`Interpolated`](Self::Interpolated), and a `None` (variant
mismatch) becomes [`Incompatible`](Self::Incompatible) carrying both
endpoints. `t` follows the interpolation contract: normalized progress
in `[0, 1]`, not clamped.
