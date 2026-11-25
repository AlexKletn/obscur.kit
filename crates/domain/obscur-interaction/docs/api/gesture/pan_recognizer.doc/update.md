Feeds a new sample at `at` (logical time `micros`) and returns the pan
delta `(dx, dy)` to apply, or [`None`] while the pan has not begun.

The sample that *crosses* the slop returns the delta from the **origin**,
not from the previous sample, so no part of the finger's travel is
swallowed by the threshold: content that starts moving on the crossing
sample lines up with the finger instead of lagging it by one slop radius.

A terminal recognizer ignores samples and returns [`None`].
