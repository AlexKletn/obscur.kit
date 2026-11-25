`SocketTransport`: the server-side driver that runs a [`DevToolsEndpoint`]
over a real local socket (spec §7.6 local socket transport, §7.7 security).

This is the real counterpart to the in-process
[`LocalTransport`](crate::transport::LocalTransport): instead of moving typed
values between two [`VecDeque`](std::collections::VecDeque)s, it reads framed
[`WireCodec`](crate::transport::WireCodec) messages off a
[`SocketStream`], feeds each into the *unchanged* endpoint (handshake →
authorize → dispatch → audit), and writes the endpoint's typed response back.
The transport never bypasses authentication, never grants a scope, and
enables nothing: a fresh endpoint is still disabled by default and rejects
every handshake until the host calls [`enable`](DevToolsEndpoint::enable)
(spec §7.7).

It writes exactly one bit of session state, and only
[`serve_with_events`](SocketTransport::serve_with_events) does: that this
session's events are being serviced, which is a fact about the loop the
caller is standing in rather than a permission. It cannot loosen an
authorisation — a `Subscribe` is still checked against the granted
[`ScopeSet`](obscur_devtools_protocol::message::ScopeSet) first, and this bit
only decides whether a scope-authorised subscription is honoured or denied.

A single [`serve`](SocketTransport::serve) call handles one connected client
session in its own thread; the endpoint's session state (negotiated version,
granted scopes, subscriptions, audit log) persists across the messages of
that session. At most [`MESSAGE_LIMIT`] messages are served per connection,
honouring the §7.7 message limit the in-process transport enforces on its
queue depth.
