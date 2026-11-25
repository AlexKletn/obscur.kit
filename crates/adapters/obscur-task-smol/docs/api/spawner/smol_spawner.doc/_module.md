A smol-backed [`Spawner`] + [`BlockingExecutor`] adapter (spec §2.7 optional
task ports — "Опциональные реализации task ports. Ни одна не является core
dependency.").

[`SmolSpawner`] implements the executor-agnostic [`Spawner`] port on top of
[`async_executor::Executor`]. Unlike an ambient runtime, that executor does
nothing until a thread drives it, so this adapter owns a worker thread that
runs the executor until the spawner is dropped. The kit's core never names
smol; this adapter is one concrete way a host wires the port.

It also implements [`BlockingExecutor`] (spec §3.3), on smol's *separate*
blocking thread pool rather than on that worker — see the impl for why the
two must not share a thread. That impl is what lets a `UiRuntime` run its
resource loads off the frame (§1.5 "resource loading" as parallel work):
before it, the only `BlockingExecutor` in the workspace was the headless
`DeterministicExecutor`, which runs every job inline on the caller by
design, so nothing could put a real thread behind the port.
