A deterministic test host [`NetworkService`] backed by canned routes
(spec §5.15).

`StubNetworkService` maps an exact request URL to a pre-built
[`NetworkResponse`] and records every request it receives, with no real
I/O, clock, or randomness. A [`fetch`](NetworkService::fetch) returns a
clone of the response routed for the request's URL, or
[`NetworkError::InvalidUrl`] when no route matches. It reports
[`SupportLevel::Emulated`] because it emulates a host loader rather than
reaching a real network.
