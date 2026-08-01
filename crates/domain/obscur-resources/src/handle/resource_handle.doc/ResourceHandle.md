A cheap, `Copy` identifier for a resource managed by the Resource Hub
(spec §5.15).

A `ResourceHandle` pairs a [`ResourceId`] with a [`ResourceKind`] tag; it is
a pure identifier that owns no data and no state (load lifecycle state lives
in [`crate::state`], keyed by handle in the hub). The typed handles such as
[`ImageHandle`] are distinct newtypes that convert to and from this generic
handle, giving compile-time type-safety while sharing one identity space.
