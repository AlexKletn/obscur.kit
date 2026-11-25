`SocketListener`: a local-only listening socket for DevTools connections
(spec §7.6 local socket transport, §7.7 local-only).

The listener binds either a Unix domain socket (preferred on Unix) or a
`127.0.0.1` loopback TCP socket (the portable fallback) and accepts client
connections, handing each back as a framed [`SocketStream`]. It never binds a
routable interface: TCP is pinned to loopback and the Unix variant is a
filesystem path, keeping the endpoint local-only per §7.7. Enabling and
authenticating a session remains entirely the endpoint's responsibility — the
listener only moves bytes.
