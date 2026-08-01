A deterministic, single-threaded executor: jobs run in strict FIFO order,
exactly when the test drives them — never on a background thread and never
reordered.

Jobs are enqueued by [`spawn`](Self::spawn) (or via the
[`BlockingExecutor`] port) and only run when the test calls
[`run`](Self::run) or [`run_next`](Self::run_next). Because a job may itself
`spawn` more jobs, the internal queue lives behind a [`Mutex`] and jobs are
popped out of the guard before they are invoked ("pop-then-run"), so a job
that spawns during its own run never deadlocks on the lock it already holds.

The queue is a [`Mutex`] rather than a `RefCell`, and jobs are required to be
[`Send`], purely so this type satisfies the `BlockingExecutor + Send + Sync`
bound on
[`UiRuntime::set_blocking_executor`](obscur_runtime::UiRuntime::set_blocking_executor).
That bound is what a real thread-pool adapter needs; without it the *only*
executor a headless test could install would be a hand-rolled double, and the
deterministic one — the one §7.4 actually specifies — could not be wired to a
runtime at all. Nothing here ever touches a second thread.

This is the Phase 1 model of controlled execution using [`FnOnce`] jobs. A
full [`Future`](std::future::Future)-based spawner (polling with a waker) is
a later addition; here "blocking" work is simply queued and run inline when
the test drives it, so there is never any real blocking or thread.
