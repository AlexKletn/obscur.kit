A [`BlockingExecutor`] that runs every job on a freshly spawned thread and
joins it — the "parallelism available" adapter answer, with the completion
order still under the caller's control.
