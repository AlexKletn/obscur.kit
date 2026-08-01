A fire-and-forget wake [`Timer`] backed by smol (spec §2.7 optional task
ports, §3.5 timer facility).

Each [`SmolTimer::schedule`] spawns a detached task onto the shared executor
that awaits an [`async_io::Timer`] for the delay and then calls the stored
wake action. The executor must be the one a
[`SmolSpawner`](crate::spawner::SmolSpawner) drives — obtained from
[`SmolSpawner::executor`](crate::spawner::SmolSpawner::executor) — so the
scheduled task actually runs. Because the port has no callback parameter, the
action is fixed at construction. The timer performs no coalescing or
de-duplication: every `schedule` arms an independent wake, and collapsing
redundant wakes is the scheduler's responsibility.
