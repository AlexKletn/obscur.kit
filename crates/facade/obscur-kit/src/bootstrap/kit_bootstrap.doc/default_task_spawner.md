Builds the default task spawner (a smol-backed executor).

The returned object satisfies **two** ports: [`Spawner`](crate::task::Spawner),
for `Send` futures on its own worker thread, and
[`BlockingExecutor`](crate::task::BlockingExecutor), for blocking jobs on
smol's blocking pool. Only the second has a consumer inside the kit —
[`install_blocking_executor`](Self::install_blocking_executor) is the
one-call way to reach it; nothing in the runtime accepts a `Spawner`, so
that half is for the host's own work.
