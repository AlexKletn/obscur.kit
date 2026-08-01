End-to-end: an external DevTools client reads a **live** runtime's real tree
over the socket (spec §7.6). A headless `UiRuntime` renders a scene, the
facade bridges it into a `SnapshotInspectSource`, a `DevToolsEndpoint` serves
it over a real loopback socket, and a `SocketClient` handshakes and inspects
— proving the observe half of the agent loop works for an external process,
not just in-process.

The lower half covers the **pushed** direction (§7-C): a subscriber over the
same real socket receives events a live runtime genuinely produced, and a host
that cannot push denies the subscription instead of granting it.
