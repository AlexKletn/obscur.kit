A concrete presentation value the motion engine can animate (spec §4.17).

`MotionValue` is the dynamic, type-erased carrier for the engine's *arbitrary
typed interpolation*: a single enum spanning the supported presentation value
kinds so timelines, tracks and drivers can move any of them through one code
path. [`Scalar`](Self::Scalar) covers every 1-D quantity (opacity, rotation,
a scale factor, a single translation axis); the geometric variants carry
whole points, sizes and rects for transforms and layout-affecting animation.

Two `MotionValue`s only blend when they hold the *same* variant. Prefer
[`interpolate_checked`](Self::interpolate_checked), which reports a variant
mismatch as `None`; the [`Interpolatable`] impl instead degrades gracefully
to a discrete swap (see its documentation) because it must return `Self`.
