One resource's complete record inside the [`crate::hub::ResourceHub`]
(spec §5.15).

An entry binds a resource's identity ([`ResourceHandle`]), its origin
([`ResourceSource`]) and category ([`ResourceKind`]), its owning
[`ResourceNamespace`] scope, its current load lifecycle [`ResourceState`], the
authoritative content [`ResourceVersion`], and a `ref_count` recording how many
times the deduplicated resource has been requested.

The `version` field is the linchpin of **stale-job rejection**: it is the
version the hub currently considers authoritative. A load job captures this
value when it begins and presents it back on completion; if a hot reload has
since advanced `version`, the returning job is stale and is rejected (see
[`crate::hub::ResourceHub::complete`]).
