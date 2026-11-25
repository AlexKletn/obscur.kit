Enqueues the (already-boxed) blocking `job`.

This executor is deterministic and single-threaded: "blocking" work is
just queued and run inline when the test drives execution via
[`run`](Self::run) / [`run_next`](Self::run_next). There is never any
real blocking or a background thread.
