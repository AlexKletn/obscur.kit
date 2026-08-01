Samples `track` at normalized progress `t`, returning the presentation
[`MotionValue`] to overlay (spec §4.17).

`t` is clamped to `[0, 1]`. Returns [`None`] only for an **empty** track
(no keyframes to present). Progress at or before the first keyframe holds
the first value; at or after the last keyframe holds the last value.
Between two keyframes the local progress is eased by the lower keyframe's
[`Easing`](obscur_motion::track::Easing) and used to blend the two values,
falling back to the upper value on a variant mismatch — exactly the
[`KeyframeTrack`] sampling rule, but surfaced as an [`Option`] so an empty
track produces no overlay rather than a degenerate zero.
