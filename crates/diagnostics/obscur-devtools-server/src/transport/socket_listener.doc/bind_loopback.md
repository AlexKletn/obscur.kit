Binds a TCP listener to an ephemeral `127.0.0.1` port.

Loopback-only: the socket is never reachable off the local host. Use
[`local_addr`](Self::local_addr) to discover the chosen port for a client
to connect to.

# Errors

Returns an [`io::Error`] if the loopback bind fails.
