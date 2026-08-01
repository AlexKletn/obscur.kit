Installs the default **threaded** blocking-work facility on `runtime`, so
its resource loads run on another thread instead of on the frame
(spec §1.5 "resource loading" as potential parallel work, §3.3 blocking
work is opt-in and capability-dependent).

This is the one call that turns §1.5's resource-loading parallelism on. It
is **opt-in by design, not by omission**: §3.3 makes blocking work a
capability, and the runtime without it is not degraded — the load simply
runs on the frame thread and the image is ready that same frame.

# What changes when you call this

A `url()` image no longer paints on the frame that first references it.
The load is handed to the pool, comes back as a typed message, and is
committed at the next frame's synchronization point — so it paints one
frame later. A host that renders exactly one frame and reads back the
pixels (an offscreen screenshot) must therefore **not** install this, or
must render again after
[`poll_resource_tasks`](crate::runtime::UiRuntime::poll_resource_tasks)
reports the load landed. A host with a live event loop should also call
`poll_resource_tasks` from it, which is what asks for a frame when a load
lands while the UI is idle; a host that renders continuously does not need
to, because the frame commits results itself.

# Determinism

Do not install this on a headless host.
[`HeadlessHost`](crate::headless::HeadlessHost) installs a
`DeterministicExecutor` on the same port and would be overwritten, taking
§7.3/§7.4's reproducibility with it: the deterministic executor's whole
job is that a load lands when `run_tasks` says so and never before.

Returns the spawner so the host can keep using its
[`Spawner`](crate::task::Spawner) half; the runtime holds its own
reference, so dropping the returned handle does not disarm the runtime.
