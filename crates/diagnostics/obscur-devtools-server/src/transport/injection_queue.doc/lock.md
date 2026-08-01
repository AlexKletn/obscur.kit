Locks the shared state, recovering from a poisoned lock rather than
propagating it: a panicked serving thread must not take the host down with
it — see [`EventQueue`](crate::transport::EventQueue)'s identical note.
