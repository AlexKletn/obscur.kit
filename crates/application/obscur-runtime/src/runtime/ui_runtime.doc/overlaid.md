The settled cascade value of every node the motion overlay has written
into `computed`, keyed by node.

[`apply_motion_overlay`](Self::apply_motion_overlay) samples *after*
inheritance, so a node it touches no longer holds the value the resolver
produced. Two readers need that value back: this frame's transition
change-detection, which must compare settled targets rather than
mid-animation samples, and an incremental restyle, which treats a retained
style as the resolver's own output. So the overlay stashes the pure style
here before overwriting it, and the style phase puts it back.

The invariant, maintained by every path through
[`run_frame`](Self::run_frame): `computed[n]` is a pure cascade value
exactly when `n` is absent here, and the entry here is that pure value.
Empty whenever nothing is animating.
