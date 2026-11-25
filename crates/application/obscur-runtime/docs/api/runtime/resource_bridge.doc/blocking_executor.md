The host-installed blocking facility (spec §3.3 "optional blocking-work
facility", "`spawn_blocking` is explicit and capability-dependent").

Absent by default, and that default is not a degradation: with no executor
a load still runs **on the frame thread** (no background thread), but its
run is **deferred to after the frame phase (frame thread, on poll)** rather
than performed inside `render_now`, so the loader is never reached while a
frame is in progress (§3.4, §5.15). What installing an executor buys is
that the load leaves the frame thread entirely — the core is tied to no
particular async runtime either way (§3.3 "the core is not tied to Tokio").
