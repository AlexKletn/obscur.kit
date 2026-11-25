Enqueues a server→client message.

# Errors

Returns [`ProtocolError::LimitExceeded`] when the client-bound queue
already holds [`MESSAGE_LIMIT`] messages.
