A [`BlockingExecutor`] that QUEUES jobs instead of running them, so a test
controls exactly when a load completes (spec §7.4 "controlled async completion").

The real smol adapter runs them on a pool; the deterministic shape is what makes
the assertions possible at all — that the frame did *not* wait for IO is only
observable if the job provably has not run yet, and "not yet" is a state only a
queue can hold.
