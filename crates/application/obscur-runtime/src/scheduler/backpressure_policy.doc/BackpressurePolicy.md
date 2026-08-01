The policy for admitting frame requests under load (spec §3.5).

The scheduler is event-driven and must stay bounded: it caps the number of
pending frames and may skip stale intermediate frames rather than let work
pile up (§3.5). This value captures those two knobs and is a pure function of
them — [`decide`](BackpressurePolicy::decide) never touches wall-clock or
randomness.
