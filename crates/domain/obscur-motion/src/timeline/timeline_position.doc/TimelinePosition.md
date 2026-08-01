Where a [`Timeline`](super::Timeline) currently is within its current
iteration (spec §4.17).

A position is a pure snapshot: how far the current iteration has run
([`elapsed`](Self::elapsed) logical seconds), the normalized point reached
([`progress`](Self::progress) in `0..=1`), and how many iterations have
already completed ([`iteration`](Self::iteration), 0-based, for repeats).

`progress` is the **raw linear** position within the iteration. Easing is
*not* applied here: tracks and interpolators downstream shape this linear
progress into eased output, keeping the timeline itself a plain integrator.

Time is measured in unspecified "logical seconds" (`f32`): the engine is
time-agnostic, so these units mean whatever the driver's `delta` supplies.
