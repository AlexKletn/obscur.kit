The half that pins the §3.3 capability default: without the executor NO
background thread is spawned — the load runs on the frame thread. What the
frame-purity ruling (§3.4/§5.15) changed is only its TIMING: the load is
deferred out of `render_now` and run by `poll_resource_tasks`, still on the
frame thread. So the loader is never touched during the frame, and when it is
finally run it runs right here on the caller — no thread was ever spawned.

This is what makes the off-thread test above discriminating: both halves drive
the identical runtime path, and the only difference is the one install call —
with it, the load leaves this thread; without it, the load stays on it.
