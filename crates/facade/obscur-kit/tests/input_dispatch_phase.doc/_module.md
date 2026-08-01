End-to-end: input is **dispatched by the frame pipeline**, not the instant a
host hands it over (spec §3.4 phase 3).

The immediate entry points (`handle_pointer` and friends) make *when* an event
is handled a property of when the host chose to call, and leave the ordering
between input, commits and frames for the host to get right. `queue_input`
moves that into the pipeline: events are received, the frame is chosen, and
only then is input dispatched — before anything commits.

The two paths share one implementation, so the interesting assertions are not
"does dispatch work" but the properties only a phase can have: nothing happens
until the frame runs, a burst is served in order by one frame, and what
dispatch invalidates is folded into that same frame rather than scheduling
another.
