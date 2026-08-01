A [`Spawner`] that schedules `Send` futures onto a smol
[`async_executor::Executor`] driven by an owned worker thread (spec §2.7
optional task ports).

The executor is shared behind an [`Arc`]; the worker thread calls
[`Executor::run`] to drive every spawned task until shutdown. Because the
executor is shared, a [`SmolTimer`](crate::timer::SmolTimer) built from
[`SmolSpawner::executor`] schedules its wake tasks onto the same running
executor, which is required for those timers to fire.
