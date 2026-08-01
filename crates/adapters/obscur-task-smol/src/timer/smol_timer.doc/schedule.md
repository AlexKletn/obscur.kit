Schedules a wake-up after `delay`.

This spawns a detached task on the shared executor that awaits an
[`async_io::Timer::after`] for `delay` and then invokes the wake action.
It is fire-and-forget: the task is not tracked, nothing is awaited, and
coalescing or de-duplicating overlapping wakes is the scheduler's job,
not this adapter's.
