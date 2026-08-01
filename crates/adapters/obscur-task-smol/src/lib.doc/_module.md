smol-based task port adapter (spec §2.7 — "Опциональные реализации task
ports. Ни одна не является core dependency.").

smol-backed implementations of the `obscur-task` ports, built directly on
the smol ecosystem's building blocks (`async-executor`, `async-io`,
`futures-lite`, `async-channel`, `blocking`) rather than the `smol` umbrella
crate. Every implementation here is optional: the kit's core never depends on
smol, and a host that wires this adapter chooses to bring the smol executor
into its reach.

Unlike an ambient runtime, an [`async_executor::Executor`] must be driven by
a thread calling its `run` method, so [`spawner::SmolSpawner`] owns a worker
thread and shares its executor with [`timer::SmolTimer`] so scheduled work
actually runs.

[`spawner::SmolSpawner`] satisfies **two** ports: `Spawner`, on that worker
thread, and `BlockingExecutor`, on smol's separate blocking pool. The kit
consumes only the second — `UiRuntime::set_blocking_executor` is the one seam
any of these ports reaches — so the `Spawner`/`Timer` halves remain for the
host's own work.
