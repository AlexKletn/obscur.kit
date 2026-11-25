Creates a spawner with a fresh executor and a worker thread that drives
it until the spawner is dropped.

# Panics

Panics if the worker thread cannot be spawned by the OS.
