Enqueues a short, nonblocking operation on the runtime owner thread.
Each idle frame poll or fairness slot uses the configured background budget.
Runnable operations execute in enqueue order and take precedence over
accounting-only units. Dropping the runtime drops pending operations.

The host must poll the runtime to service queued operations. Use the task
executor for blocking work or expensive computation.
