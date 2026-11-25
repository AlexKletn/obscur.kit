Loads the raw encoded bytes behind `source` (spec §5.15).

# Errors

Returns [`LoadedBytes`] on success, or a [`ResourceError`] describing why
the source could not be resolved — for example
[`ResourceError::SourceUnavailable`] when this loader cannot reach the
source at all.
