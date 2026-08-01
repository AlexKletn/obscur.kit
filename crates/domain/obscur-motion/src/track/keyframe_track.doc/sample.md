Samples the track at normalized progress `t` (spec §4.17).

`t` is clamped to `[0, 1]`. An empty track is a degenerate case and
yields `MotionValue::scalar(0.0)`. Progress at or before the first
keyframe yields the first value; at or after the last yields the last.
Between two keyframes the local progress is eased by the lower
keyframe's easing and used to blend the two values, falling back to the
upper value on a variant mismatch.
