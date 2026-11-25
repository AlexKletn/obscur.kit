One controlled async completion: a `value` produced for a particular
[`OperationId`].

Delivering a completion is how a headless test resolves an async result at a
deterministic synchronization point. The [`OperationId`] identifies the
operation the result belongs to, so callers can recognize and drop stale
results whose generation/version no longer matches (spec §3.3).
