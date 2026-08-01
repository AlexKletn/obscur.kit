Performs the connection handshake and returns the server's response.

# Errors

Returns an [`io::Error`] on a socket failure, or if the server's reply is
not a handshake response (a protocol violation).
