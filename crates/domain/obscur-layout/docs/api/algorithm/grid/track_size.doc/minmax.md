Builds a [`MinMax`](TrackSize::MinMax) track from a `min` and `max` bound.

Convenience so callers need not box the bounds by hand:
`TrackSize::minmax(TrackSize::Fixed(100.0), TrackSize::Fr(1.0))`.
