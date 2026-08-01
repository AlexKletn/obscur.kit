Re-enumerates the platform's monitors into `state` (spec §6.6).

This is the only place a [`MonitorHandle`](winit::monitor::MonitorHandle) is
reachable, so the pull-based `displays()` is served from what this leaves
behind. It runs on every loop turn rather than once at startup because winit
0.30 has no monitor notification to run on instead — a topology refreshed
only at `resumed` would still describe an unplugged monitor. The re-read is
what it costs to not be stale, and the cost is platform-shaped: X11 and
Wayland answer from winit's own cached output list, while Windows
re-enumerates through `EnumDisplayMonitors` on every call.
[`BackendState::record_displays`] makes the write idempotent, so an unchanged
topology enqueues nothing.
