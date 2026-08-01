Logical seconds for one iteration. Always strictly positive: a `0` or
negative duration passed to [`new`](Self::new) is clamped up to
[`MIN_DURATION`](Self::MIN_DURATION) to keep progress division safe.
