An in-process, loopback message channel between a DevTools client and server.

This models the spec's *local socket* transport (spec §7.6) without any real
IO: two bounded queues carry [`ServerBound`] messages one way and
[`ClientBound`] messages the other. A real socket transport would expose the
same shape (`send_to_server` / `poll_server` / `push_to_client` /
`poll_client`) over a kernel socket. Both queues enforce
[`MESSAGE_LIMIT`](obscur_devtools_protocol::message::MESSAGE_LIMIT):
enqueueing onto a full queue fails with
[`ProtocolError::LimitExceeded`] rather than growing unboundedly (spec §7.7).
