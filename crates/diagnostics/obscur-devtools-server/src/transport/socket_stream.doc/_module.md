`SocketStream`: a length-prefixed framed byte channel over a real local
kernel socket (spec §7.6 local socket transport).

This is the low-level connection shared by the server-side
[`SocketTransport`](crate::transport::SocketTransport) and the client-side
[`SocketClient`](crate::transport::SocketClient). It wraps either a Unix
domain stream (preferred on Unix) or a `127.0.0.1` TCP stream (the portable
fallback) and frames the raw message bodies produced by
[`WireCodec`](crate::transport::WireCodec) as a `u32` little-endian length
prefix followed by that many payload bytes.

Frames are bounded by [`MAX_FRAME_LEN`]: a declared length above the cap is
refused before any allocation, so a peer cannot drive unbounded memory use
(spec §7.7 limits). The socket is only ever bound to a loopback address or a
filesystem path — never a public interface.
