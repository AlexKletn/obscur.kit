The manually-driven deterministic job runner, also installed on `runtime`
as its blocking-work port. Held as an [`Arc`] because the runtime holds
the other end.
