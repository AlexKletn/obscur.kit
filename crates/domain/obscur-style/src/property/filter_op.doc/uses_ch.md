Whether this op carries a `ch` term.

Only `drop-shadow()` carries lengths at all: `blur(<length>)`'s radius is
already an `f32` by computed-value time (the CSS lowering resolved it), so
a `blur(1ch)` is lost before this is asked — a lowering-side limitation
mirrored in `absolutize_filter`, not papered over here.
