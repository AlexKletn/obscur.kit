Reads and answers exactly one client message, driving `endpoint`.

Returns `Ok(false)` when the client has closed the connection cleanly (no
more messages) or the per-connection [`MESSAGE_LIMIT`] has been reached,
and `Ok(true)` after a message was answered. A message that does not
decode is answered with a [`ClientBound::Error`] carrying
[`ProtocolError::MalformedMessage`](obscur_devtools_protocol::error::ProtocolError::MalformedMessage)
rather than terminating the session.

# Errors

Returns an [`io::Error`] on any underlying socket read/write failure.
