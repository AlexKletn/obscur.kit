Begins a data drag from `source` (spec §3.2 provisional, §6.21).

While it is in flight, [`handle_pointer`](Self::handle_pointer) drives the
drag instead of hovering, focusing and clicking: a move tracks it across
drop targets, a release drops or cancels it. See [`DragCoordinator`] for
what is provisional about it and what is not.

Returns `false` when a drag is already in flight, having started nothing.

[`DragCoordinator`]: crate::orchestration::DragCoordinator
