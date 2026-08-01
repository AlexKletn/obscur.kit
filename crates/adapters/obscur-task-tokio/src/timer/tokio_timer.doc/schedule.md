Schedules a wake-up after `delay`.

This spawns a detached Tokio task that sleeps for `delay` and then invokes
the wake action. It is fire-and-forget: the task is not tracked, nothing is
awaited, and coalescing or de-duplicating overlapping wakes is the
scheduler's job, not this adapter's.
