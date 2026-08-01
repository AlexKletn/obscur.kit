Sets how long a [`read_frame`](Self::read_frame) waits before giving up,
or clears the timeout with [`None`].

A serving loop needs this to stay responsive to work that is not a client
message: without a timeout it blocks in `read` forever, so an event a host
wants to *push* could never be written until the client happened to send
something first — which is not a push at all.

# Errors

Returns an [`io::Error`] if the underlying socket rejects the timeout.
