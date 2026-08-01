Decides recovery for a failure in `subsystem` on retry `attempt`.

Returns [`RecoveryResult::Retried`] while `attempt < max_retries` — the
subsystem's work is retried — and otherwise [`RecoveryResult::FellBack`],
meaning the subsystem degrades (uses its fallback / last-good output). A
subsystem failure is never [`RecoveryResult::Propagated`]: there is no
component boundary to escape to.

`subsystem` is accepted for future per-subsystem policy and diagnostics
attribution; Phase 1 treats every subsystem uniformly.
