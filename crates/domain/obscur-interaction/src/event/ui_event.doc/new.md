Creates an event aimed at `target` from `origin`, seated at the
[`Target`](EventPhase::Target) phase with fresh propagation control, no
payload, an untrusted origin and a zero timestamp.

`current_target` starts equal to `target`, which is exactly true at the
target phase; a dispatcher overwrites it at every other step.
