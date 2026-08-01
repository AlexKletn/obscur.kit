Creates a time driver that spans `duration` logical seconds, starting at
`elapsed` `0` (progress `0`).

A `duration` that is zero, negative or `NaN` is clamped up to
[`MIN_DURATION`](Self::MIN_DURATION).
