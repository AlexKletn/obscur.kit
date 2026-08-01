A Tokio-backed [`Timer`] adapter (spec §2.7 optional task ports —
"Опциональные реализации task ports. Ни одна не является core dependency.";
§3.5 timer facility).

[`Timer::schedule`] carries only a delay, no callback, so the wake action is
supplied once at construction. [`TokioTimer`] schedules a detached Tokio task
that sleeps for the requested delay and then invokes that wake action; the
domain wires the action to request a frame (spec §3.5).
