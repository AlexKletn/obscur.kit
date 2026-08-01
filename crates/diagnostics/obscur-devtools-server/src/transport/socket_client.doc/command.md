Issues one command and returns the server's raw client-bound reply
(snapshot, subscription, acknowledgement, or protocol error).

# Errors

Returns an [`io::Error`] on any underlying socket failure.
