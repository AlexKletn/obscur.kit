The deterministic set of currently-running [`Timeline`]s, keyed by
[`TimelineId`] (spec §4.17).

The motion engine tracks every active timeline in one place so the
[`MotionScheduler`](super::MotionScheduler) can advance them, drop finished
ones, and report frame demand. Timelines are stored in a [`BTreeMap`] keyed by
their stable [`id`](Timeline::id), which gives a **deterministic** iteration
order (ascending by id) regardless of insertion order — a requirement for the
engine's reproducibility (spec §4.17). Feeding two sets the same operations
yields identical iteration and advancement.
