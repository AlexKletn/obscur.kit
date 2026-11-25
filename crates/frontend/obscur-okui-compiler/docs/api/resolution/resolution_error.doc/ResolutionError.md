Why name resolution failed for a single reference or declaration (spec
§2.11).

The error carries the offending `name` and the byte `offset` into the source
where the failure was detected, so a diagnostic can point at it. This type
implements [`std::error::Error`] so it composes with the standard error
ecosystem.
