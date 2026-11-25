A fault encountered while decoding a DevTools wire envelope (spec §7.6).

The variants name the ways a byte buffer can fail to be a well-formed
message: a truncated buffer, a string that is not UTF-8, or an unknown
discriminant tag. The socket driver maps every one of these to a
[`ProtocolError::MalformedMessage`] on the wire (spec §7.7).
