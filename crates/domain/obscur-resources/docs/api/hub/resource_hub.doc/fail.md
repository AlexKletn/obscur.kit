Fails a load job for `handle`, recording the error — unless the job is
stale (spec §5.15 stale-job rejection).

Applies the same version check as [`ResourceHub::complete`]: a superseded
job leaves the state untouched and returns `false`. On a match the state
becomes [`ResourceState::Failed`] carrying `error` at the current version,
and `true` is returned. An unknown handle returns `false`.
