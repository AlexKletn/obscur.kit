Drains every finished resource-load task, applying its result, and returns
how many were applied (spec §3.3, §3.5 `ResourceCompletion`).

A host calls this from its event loop. Draining a completion requests a
[`FrameReason::ResourceCompletion`] frame, so a load that finishes while the
UI is idle still gets painted — without the runtime ever waking itself from
another thread.

A **stale** completion is dropped: the hub rejects a job whose version is
behind, and the runtime caches nothing for it. That happens when a resource
was re-requested while an earlier load was still in flight, and it is why
the outcome carries the version it started under rather than trusting
arrival order.

Returns `0` — and requests nothing — when no task has finished, so calling
it every loop iteration is free.

Calling it is nonetheless **optional**: [`run_frame`](Self::run_frame)
commits finished results itself, at the top of the frame, so a host that
only ever renders still sees its loads land (§1.5 "cross-thread results …
are committed at deterministic synchronization points"). This method is
what lets a load land while the UI is otherwise *idle* — it is the only
path that asks for a frame.
