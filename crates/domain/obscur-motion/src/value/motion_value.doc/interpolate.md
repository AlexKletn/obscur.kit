Blends toward `other`, degrading gracefully on a variant mismatch.

When both operands hold the same variant this delegates to
[`interpolate_checked`](MotionValue::interpolate_checked) and blends the
inner value component-wise. Because this trait method must return `Self`
(not `Option`), a *mismatch* cannot be signalled; instead it performs a
discrete swap — `self` while `t < 0.5`, otherwise `other` — so the
animation still resolves to a valid endpoint. Callers that need to
detect incompatibility should use `interpolate_checked` or
[`InterpolationResult`](super::InterpolationResult) instead.
