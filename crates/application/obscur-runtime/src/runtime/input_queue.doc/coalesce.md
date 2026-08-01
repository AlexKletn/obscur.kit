Folds a frame request for `reason` into the running frame's plan, reporting
whether it did.

`false` when no dispatch phase is running, which is the caller's signal to
route the request to the scheduler as usual.
