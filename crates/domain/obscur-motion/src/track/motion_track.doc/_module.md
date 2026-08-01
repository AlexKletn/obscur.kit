The [`MotionTrack`] track port and the shared [`Easing`] helper (spec §4.17).

A *track* turns a normalized timeline progress `t` into a presentation
[`MotionValue`]. The timeline upstream produces only raw linear progress;
tracks are where easing curves and value interpolation are applied, so the
track is the compile target for CSS `transition` / `@keyframes` / spring
`animation` once the timeline has been separated out. [`Easing`] lives here
because it is shared by both the transition and keyframe tracks.
