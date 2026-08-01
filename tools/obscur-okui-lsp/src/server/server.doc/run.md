Runs the message loop until an `exit` notification or end of stream.

Reads `Content-Length`-framed JSON-RPC messages from `reader`, dispatches
each, and writes framed responses/notifications to `writer`.

# Errors

Propagates I/O errors from the transport.
