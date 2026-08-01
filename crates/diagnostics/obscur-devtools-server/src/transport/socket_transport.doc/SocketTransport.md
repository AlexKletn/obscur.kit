Serves a [`DevToolsEndpoint`] over one framed local-socket connection.

Constructed from an accepted [`SocketStream`]
(see [`SocketListener::accept`](crate::transport::SocketListener::accept)),
it owns the connection for the lifetime of a single client session.
