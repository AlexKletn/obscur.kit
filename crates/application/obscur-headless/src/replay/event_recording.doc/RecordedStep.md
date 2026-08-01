One entry in an [`EventRecording`]: a monotonic `tick` and its `event`.

`tick` is either a monotonic sequence index or the deterministic clock value
at record time; the recording preserves entries in the order they were added.
