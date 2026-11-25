The outcome of the last drag to finish, taken once.

This is how a drop reaches the host: `handle_pointer` ends the drag as part
of dispatching the release, so the outcome is parked here rather than
squeezed into [`PointerDispatch`] — which every non-dragging caller would
then have to ignore. Returns [`None`] when no drag has finished since the
last call.

[`PointerDispatch`]: crate::PointerDispatch
