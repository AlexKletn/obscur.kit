§2.4's pointer **delta**: how far this pointer moved since its own previous
event, in logical pixels.

§2.4 requires a pointer event to carry *"logical position and delta"* — two
facts, not one. The position is where the pointer is; the delta is how it got
there, and it is not derivable from two successive positions in every case a
handler cares about. A relative device (a trackpad, a captured mouse) can
report motion the position does not show once the cursor is clamped at the edge
of the screen, and a coalesced move reports the *sum* of several motions at a
single position.

# Why this is not `obscur_geometry::Vector`

Because `Vector` is still a skeleton stub (`pub(crate) struct Vector;`) with no
contract implemented. Modeling the delta locally keeps §2.4's field honest
today rather than blocking it on a foundation type; when `Vector` lands this
becomes a type alias or a `From` pair, and no caller has to change shape
because the accessors already read `x` / `y`.

A pointer that did not move — a press, a release, an enter — carries
[`ZERO`](Self::ZERO), which is the truth rather than a placeholder: nothing
moved.
