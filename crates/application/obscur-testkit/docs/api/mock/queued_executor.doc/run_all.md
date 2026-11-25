Runs every queued job, emptying the queue.

# Panics

If the queue lock is poisoned — which only happens if a job already panicked on
another thread, and that panic is the failure worth reporting.
