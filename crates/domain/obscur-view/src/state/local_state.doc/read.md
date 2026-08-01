Produces a read-only [`ReadSignal`] snapshot of the current value and
generation.

This is the only way state leaves `LocalState`, and it is read-only:
consumers receive a value they cannot write back through. Re-calling
`read` after a mutation yields a snapshot with a newer generation.
