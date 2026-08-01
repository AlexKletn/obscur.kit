Returns a handle to the shared executor.

Cloning the [`Arc`] lets a [`SmolTimer`](crate::timer::SmolTimer) schedule
wake tasks onto the same executor this spawner drives, which is required
for those timers to run.
