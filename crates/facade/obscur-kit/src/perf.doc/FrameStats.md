A rolling window of the most recent [`FrameSample`]s, with FPS and timing
summaries derived from them.

The window is a fixed-capacity ring: once full, each new sample evicts the
oldest, so the stats always describe the recent past.
