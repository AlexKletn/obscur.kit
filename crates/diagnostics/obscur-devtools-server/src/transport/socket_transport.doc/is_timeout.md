Whether `error` is a read that simply timed out rather than a broken socket.

Platforms disagree on which kind a socket read timeout reports, so both are
accepted; treating either as fatal would end a healthy session on its first
quiet interval.
