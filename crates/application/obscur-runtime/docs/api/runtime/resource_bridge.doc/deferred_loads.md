Resource loads the frame phase QUEUED but has not yet run, when **no**
[`blocking_executor`](Self::blocking_executor) is installed (spec §3.3).

The §3.3 no-executor default is preserved — the load still runs on the
frame thread, spawning no background thread — but its *timing* is lifted
out of the synchronous frame phase: the loader must never be invoked while
a frame is in progress (§3.4 "the synchronous frame phase must stay short",
§5.15 async load). Each closure carries its own loader/decoder and yields
the same typed [`ResourceOutcome`] the executor path returns. Drained by
[`run_deferred_loads`](Self::run_deferred_loads) on the frame thread, AFTER
the frame phase. The dedupe that stops a load re-queuing —
[`image_handles`](Self::image_handles) — is set at ENQUEUE time, so a
resource already queued (or already failed) is never queued twice.
