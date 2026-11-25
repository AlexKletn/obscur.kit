Enqueues `job` to run later. This does **not** run the job; call
[`run`](Self::run) or [`run_next`](Self::run_next) to drive execution.

`job` must be [`Send`] because the same queue backs the
[`BlockingExecutor`] port, whose jobs are `Send`. Nothing is ever sent
anywhere: the job runs on the calling thread, when the test says so.
