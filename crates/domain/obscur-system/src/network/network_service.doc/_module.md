The host-provided port for satisfying network fetches (spec §5.15).

The toolkit performs **no network I/O of its own**: [`NetworkService`] is a
port a host/application implements to resolve a
[`NetworkRequest`](crate::network::NetworkRequest) into a
[`NetworkResponse`](crate::network::NetworkResponse) through its own loader
(spec §5.15). The honest kit default — [`NullNetworkService`] — provides
nothing, enforcing the no-network policy; [`StubNetworkService`] is a
deterministic test host with canned routes.
