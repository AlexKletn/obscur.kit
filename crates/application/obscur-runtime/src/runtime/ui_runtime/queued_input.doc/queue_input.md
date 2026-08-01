Queues `event` for the frame pipeline's **dispatch phase** and requests the
frame that will run it (spec §3.4 phases 1 and 3).

# Why queue rather than dispatch now

[`handle_pointer`](Self::handle_pointer) and its siblings dispatch the
instant they are called, which makes *when* an event is handled a property
of when the host chose to call — and makes the ordering between input,
state commits and frames the host's to get right. Queuing moves that into
the pipeline: events are received (phase 1), the frame is chosen (phase 2),
and only then is input dispatched (phase 3), before anything commits.

The two paths share one implementation — the queue dispatches *through*
`handle_pointer` and friends — so an event handled either way does exactly
the same thing. Only the timing differs, and the immediate entry points
remain for hosts that want it.

A burst queued between frames is dispatched in order by the single frame
that serves it, and whatever that dispatch invalidates is folded into that
same frame's plan rather than scheduling another.
