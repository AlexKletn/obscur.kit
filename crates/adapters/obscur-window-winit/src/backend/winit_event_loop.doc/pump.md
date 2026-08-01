Materializes deferred windows, applies queued operations, and refreshes
the display topology (spec §6.1, §6.6).

Runs on every loop turn that has an [`ActiveEventLoop`]: it re-enumerates
the monitors, drains the pending-create queue (building each real winit
window and emitting a [`WindowEvent::Created`]), pushes queued property
sets onto live windows, grants queued focus, and drops closed windows.
