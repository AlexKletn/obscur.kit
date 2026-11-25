Enqueues a client→server message.

# Errors

Returns [`ProtocolError::LimitExceeded`] when the server-bound queue
already holds [`MESSAGE_LIMIT`] messages.
