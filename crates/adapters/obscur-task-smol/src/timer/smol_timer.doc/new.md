Creates a timer that, for each scheduled delay, wakes by calling `wake`
on a task spawned onto `executor`.

Pass the executor of the [`SmolSpawner`](crate::spawner::SmolSpawner)
driving this timer's tasks so scheduled wakes actually run.
