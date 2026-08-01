Turns the overlay on or off, reporting whether that **changed** anything.

The caller schedules the frame that makes the change visible: this entity
does not hold the scheduler, so it reports the edge and
[`UiRuntime::set_inspect_mode`](super::UiRuntime::set_inspect_mode) acts on
it.
