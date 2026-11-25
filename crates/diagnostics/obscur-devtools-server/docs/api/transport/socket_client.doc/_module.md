`SocketClient`: the client end of a real DevTools socket connection
(spec §7.6 local socket transport).

A DevTools client process (CLI/GUI) connects to a [`SocketListener`]'s
address, presents a handshake, and issues capability-scoped commands. This
type is the minimal, std-only counterpart to
[`SocketTransport`](crate::transport::SocketTransport): it frames a
[`ServerBound`] with [`WireCodec`], writes it, and reads back the framed
[`ClientBound`] reply. It carries no authority of its own — the server's
endpoint decides what every message is allowed to do (spec §7.7).

[`SocketListener`]: crate::transport::SocketListener
