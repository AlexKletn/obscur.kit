Drives all queued deterministic jobs to completion and applies whatever
they finished, returning how many resource completions were applied.

Background work never runs on its own in a headless run: the runtime
queued each load on this host's executor, and it runs in FIFO order
exactly when the test calls this. Draining the finished loads into the
runtime is the second half — without it the jobs would run but their
results would never reach a frame.
