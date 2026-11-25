Rung when an image lands, so the loop's own wait ends.

Without it the loop would have to poll for readiness, which is the busy loop
§3.5 forbids. `Send + Sync` because it is called from the acquire thread.

The same shape as `LoopWaker` and the appearance watcher's callback, and for the
same reason: the thread with news is never the thread that owns the loop.
