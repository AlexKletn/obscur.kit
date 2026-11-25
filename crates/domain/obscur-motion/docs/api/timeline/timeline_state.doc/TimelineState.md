The lifecycle state of a [`Timeline`](super::Timeline) (spec §4.17).

A timeline moves `Idle` → `Running` when started, may toggle between
`Running` and `Paused` while it plays, and settles in `Finished` once all its
iterations are consumed. Because the engine is time-agnostic and
event-driven, these transitions are driven by explicit calls
(start/pause/resume/advance), never by a clock.
