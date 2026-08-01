Clones the handle so one side can read while another writes.

Both handles refer to the same kernel socket, which is full-duplex: a push
written on the clone lands even while the original is parked in `read`.

# Errors

Returns an [`io::Error`] if the underlying socket cannot be duplicated.
