Lowering `@keyframes` into native `obscur-motion` keyframe tracks (§4.17).

[`KeyframeCompiler`] turns a parsed `@keyframes` rule into a
[`CompiledKeyframes`]: one [`KeyframeTrack`] per animated property. Each
keyframe selector (`from`, `to`, or a percentage) becomes a normalized offset
in `[0, 1]`, and each declaration whose value lowers to a [`MotionValue`]
contributes a [`Keyframe`] to its property's track.

Only declarations that name one of the modelled [`PropertyId`] longhands and
whose value lowers to a scalar `MotionValue` (a number, a `px` length, or a
percentage) are animated; anything else is skipped, because a non-animatable
value has no place on a track.
