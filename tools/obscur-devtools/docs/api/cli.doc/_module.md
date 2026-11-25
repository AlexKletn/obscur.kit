The DevTools **client** interface: connect, handshake, and issue
capability-scoped inspect/subscribe commands, formatting the typed replies
for a terminal (spec §7.6).

Every function here is transport-agnostic above a connected
[`SocketClient`]: `main` wires one to a real socket, tests wire one to an
in-process loopback listener. The client carries no authority — the server's
endpoint decides what each message may do (spec §7.7).
