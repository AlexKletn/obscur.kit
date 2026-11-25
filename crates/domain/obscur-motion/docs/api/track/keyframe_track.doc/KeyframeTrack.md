An ordered list of [`Keyframe`]s sampled by normalized progress (spec §4.17).

This is the native compile target of CSS `@keyframes`: a set of value stops
at normalized offsets, with per-segment easing. The keyframes are kept sorted
by offset so sampling can walk them directly. Sampling clamps progress to the
first/last stop, then linearly locates the bracketing pair and blends the
lower value toward the upper one using the lower keyframe's easing. A blend
between mismatched [`MotionValue`] variants is impossible and falls back to
the upper keyframe's value.
