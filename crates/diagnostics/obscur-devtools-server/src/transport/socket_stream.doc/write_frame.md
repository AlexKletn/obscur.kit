Writes one framed message body: a `u32` little-endian length prefix
followed by `body`.

# Errors

Returns an [`io::Error`] if `body` exceeds [`MAX_FRAME_LEN`] or the
underlying socket write fails.
