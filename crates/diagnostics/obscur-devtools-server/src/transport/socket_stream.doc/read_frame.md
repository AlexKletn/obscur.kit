Reads one framed message body, or `None` on a clean end-of-stream.

Returns `Ok(None)` only when the peer closed the connection cleanly at a
frame boundary (no bytes of a length prefix were available).

# Errors

Returns an [`io::Error`] on a truncated frame, a length above
[`MAX_FRAME_LEN`], or any underlying socket read failure.
