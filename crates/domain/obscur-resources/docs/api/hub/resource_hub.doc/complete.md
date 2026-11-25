Completes a load job for `handle`, marking the resource ready — unless the
job is stale (spec §5.15 stale-job rejection).

If `job_version` no longer matches the resource's authoritative version (a
hot reload superseded it), the state is left untouched and `false` is
returned. On a match the state becomes [`ResourceState::Ready`] carrying
`metadata` at the current version, and `true` is returned. An unknown
handle returns `false`.
