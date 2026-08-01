Accepts and serves exactly one client session, then returns.

A fresh [`DevToolsEndpoint`] is built for the connection (its own session
state), authenticating `token` with `scopes`, enabled, and driven over the
real socket by [`SocketTransport::serve`] until the client disconnects.

# Errors

Returns an [`io::Error`] on any accept/serve socket failure.
