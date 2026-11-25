One end of a framed, local-only DevTools socket connection.

Reads and writes whole [`WireCodec`](crate::transport::WireCodec) message
bodies, delimiting each with a `u32` little-endian length prefix.
