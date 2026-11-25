The two ends of the [`InputQueue`](crate::runtime::InputQueue): the host call
that fills it, and the frame phase that drains it.

They are one subject because the queue's whole point is the gap between them.
`queue_input` moves the *timing* decision out of the host — an event queued is
handled at §3.4 phase 3, before anything commits and before the frame has chosen
its plan — and `dispatch_queued_input` is that phase, dispatching each event
through the very same entry points an immediate caller would have used. Only the
timing differs; the handling is identical by construction.

The phase is also where the frame's plan picks up what dispatch invalidated: the
latch `begin_dispatch` raises redirects a handler's own `request_frame` into
*this* frame's plan, and `end_dispatch` lowers it again. That is why a burst of
input costs one frame rather than one frame per event. What the dispatch left
behind for the caller to read is `pending_work.rs`'s `take_input_dispatches`.
