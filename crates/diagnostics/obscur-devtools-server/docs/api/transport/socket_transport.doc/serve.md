Drives `endpoint` until the client disconnects or the message limit is
reached, answering each message in turn.

# Errors

Returns an [`io::Error`] on any underlying socket read/write failure.
