Locks the shared state, recovering from a poisoned lock rather than
propagating it: a panicked serving thread must not take the host down with
it — the queue's invariant is just "these are the pending events".
