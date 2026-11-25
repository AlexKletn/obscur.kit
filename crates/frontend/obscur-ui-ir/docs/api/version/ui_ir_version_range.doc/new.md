Creates a range spanning `[min, max]` inclusive.

If `min > max` the input is inverted; `max` is clamped up to `min`, so
the result is the single-version range `[min, min]`.
