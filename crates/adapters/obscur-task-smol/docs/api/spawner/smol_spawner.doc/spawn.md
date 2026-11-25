Spawns `future` onto the executor.

[`Executor::spawn`] returns a `Task` handle; it is immediately
[`detach`](async_executor::Task::detach)ed so the work runs
fire-and-forget on the worker thread and its completion is not awaited
here.
