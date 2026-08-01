Performs the connection handshake, requesting the current protocol version
with `token`.

# Errors

Returns an [`io::Error`] on a socket failure or a protocol violation.
