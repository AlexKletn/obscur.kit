The paths the runtime asked for, in order.

# Panics

If the recording lock is poisoned — which only happens if a load already panicked
on another thread, and that panic is the failure worth reporting.
