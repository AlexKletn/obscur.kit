Reads one **unsolicited** message the server pushed, without sending
anything (spec §7.6 live inspectable state).

Every other method here writes then reads; a pushed event has no request to
pair with, so it needs its own read. Returns [`None`] when the server closed
the connection.

# Errors

Returns an [`io::Error`] on a socket failure or an undecodable frame.
