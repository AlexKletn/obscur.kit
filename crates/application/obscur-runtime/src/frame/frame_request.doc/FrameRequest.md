A request to schedule a frame for a given [`FrameReason`].

A request records only *why* a frame is wanted. Turning a stream of requests
into actual frames — coalescing several requests into one frame, and ordering
them by priority — is the scheduler's job in a later phase; a `FrameRequest`
itself carries no timing or priority state.

The type is a plain value: `Copy`, equatable, and hashable, so pending
requests can be deduplicated in a set.
