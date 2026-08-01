Sets how long a [`receive`](Self::receive) waits before giving up.

# Errors

Returns an [`io::Error`] if the underlying socket rejects the timeout.
