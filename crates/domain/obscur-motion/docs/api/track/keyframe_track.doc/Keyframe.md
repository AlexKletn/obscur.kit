One stop in a [`KeyframeTrack`]: a value pinned at a normalized offset
(spec §4.17).

`offset` is the position within the track's `[0, 1]` progress range at which
`value` is reached. Following CSS `@keyframes` semantics, `easing` is the
timing function used to interpolate *from this keyframe to the next one*, not
into this one.
