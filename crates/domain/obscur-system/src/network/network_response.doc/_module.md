The host loader's response to a network request (spec §5.15).

A [`NetworkResponse`] is the pure, backend-agnostic result a host-provided
loader returns for a [`NetworkRequest`](crate::network::NetworkRequest): an
HTTP status code, ordered response headers, and a body. The kit models it as
a value with no live connection behind it (spec §5.15).
