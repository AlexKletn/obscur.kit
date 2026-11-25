Compiles one `animation` shorthand layer into an [`AnimationRef`], or [`None`]
when the layer names no animation.

The first keyword that is not a recognised timing-function/animation keyword
is taken as the animation name. Per css-animations-1 §3.10, "the first value
in each `<single-animation>` that can be parsed as a `<time>` is assigned to
animation-duration, and the second value … is assigned to animation-delay";
the first `<number>`/`infinite` is the iteration count, and the direction and
fill-mode keywords are recognised by name.
