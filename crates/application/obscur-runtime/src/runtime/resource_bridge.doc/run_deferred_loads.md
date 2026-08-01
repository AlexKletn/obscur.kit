Runs every resource load the frame phase deferred (the no-executor path,
spec §3.3) and commits its outcome, returning how many were accepted.

A deferred load runs on the frame thread but strictly AFTER the synchronous
frame phase — the loader is never reached while a frame is in progress
(§3.4, §5.15). No background thread is spawned: the §3.3 capability default
keeps its "frame thread, not a background thread" guarantee; only the
timing moved out of `render_now`. Because the resolve pass marks a URL
pending in [`image_handles`](Self::image_handles) at ENQUEUE time, a load
already queued (or already failed) is never re-queued on a later frame, so
draining once runs each load exactly once.
