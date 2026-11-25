Signals shutdown so the worker's [`Executor::run`] future resolves, then
joins the worker thread so no thread is leaked.

Sending on the bounded channel (or its later drop) completes the recv the
worker awaits, which ends `run` and lets the thread finish; the join then
returns promptly rather than blocking.
