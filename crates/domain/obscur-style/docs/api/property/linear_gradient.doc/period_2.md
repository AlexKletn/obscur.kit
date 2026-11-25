Returns the tiling period of a repeating gradient, or [`None`] for a
non-repeating one.

Repeats with authored positions may resolve their period only at paint time.
Use [`Self::repeat_positions`] to read those positions.
