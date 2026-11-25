The command was authorized, but this host has nothing that can receive an
injected event, so it will not pretend to have delivered one (spec §7.6).

Distinct from [`Unauthorized`](Self::Unauthorized) on purpose: the client
holds the `Inject` scope and the endpoint says so. What is missing is a
runtime on the other side — a host serving inspect-only state (the
`obscur-devtools serve` demo host has no runtime at all) has nowhere to
put a synthetic click. Telling the client "unauthorized" would send it
hunting for a token it already has.
