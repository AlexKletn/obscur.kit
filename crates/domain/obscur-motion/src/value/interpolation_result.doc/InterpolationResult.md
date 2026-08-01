The typed outcome of blending two [`MotionValue`] endpoints (spec §4.17).

Interpolation only succeeds when both endpoints hold the same
[`MotionValue`] variant. This enum makes that outcome explicit: a successful
blend is [`Interpolated`](Self::Interpolated); a variant mismatch — two
endpoints that cannot be blended — is reported as
[`Incompatible`](Self::Incompatible), which preserves both endpoints so a
caller can fall back (for example to a discrete swap) without losing
information.
