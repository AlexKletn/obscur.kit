The reason a frame is being produced (spec §3.5).

Every frame is produced for exactly one reason, which lets the scheduler
coalesce and prioritise requests and lets diagnostics attribute work. The
set is closed and enumerated here; there is no string dispatch.

The type is a plain value: `Copy`, equatable, and hashable, so it can be used
as a map key or accumulated in a set of pending reasons.
