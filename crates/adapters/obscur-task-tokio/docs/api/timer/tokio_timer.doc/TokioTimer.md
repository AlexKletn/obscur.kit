A fire-and-forget wake [`Timer`] backed by Tokio (spec §2.7 optional task
ports, §3.5 timer facility).

Each [`TokioTimer::schedule`] spawns a detached task on the Tokio [`Handle`]
that sleeps for the delay and then calls the stored wake action. Because the
port has no callback parameter, the action is fixed at construction. The timer
performs no coalescing or de-duplication: every `schedule` arms an independent
wake, and collapsing redundant wakes is the scheduler's responsibility.
