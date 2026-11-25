Why a type check failed for an expression (spec §2.9).

The error records the [`kind`](TypeError::kind) of failure, the `expected`
and `found` [`ValueType`]s, and the byte `offset` into the source where the
failure was detected, so a diagnostic can point at it. This type implements
[`std::error::Error`] so it composes with the standard error ecosystem.
