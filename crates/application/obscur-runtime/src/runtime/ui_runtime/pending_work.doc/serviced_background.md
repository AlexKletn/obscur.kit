Returns how many background task-work units have been serviced through
the drain path so far.

This advances only when [`run_due_frame`](Self::run_due_frame) promotes
and runs a starved background unit, so it is the observable proof that
background work made progress despite higher-priority frames (§3.6).
