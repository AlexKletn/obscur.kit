`WireCodec`: a std-only byte codec for the DevTools wire envelopes, and its
colocated [`WireError`] (spec §7.6 local socket transport).

The protocol messages ([`ServerBound`]/[`ClientBound`]) are ordinary Rust
values with no serialization of their own; the in-process
[`LocalTransport`](crate::transport::LocalTransport) simply moves them
between queues. To carry the *same* messages over a real kernel socket this
codec turns them into bytes, mirroring the deterministic little-endian layout
the kit uses elsewhere (see `obscur-ui-ir`'s codec):

- All multi-byte integers are **little-endian** (`to_le_bytes`).
- Enums are tagged with a leading `u8` discriminant.
- Strings are a `u32` byte-length prefix followed by their UTF-8 bytes.
- `ScopeSet` is a single `u8` bitset assembled through the public
  `contains`/`with` API (bit0 `Read`, bit1 `Subscribe`, bit2 `Control`,
  bit3 `Inject`).

The codec produces the raw message body only; frame delimitation (the
length prefix) is added by [`SocketStream`](crate::transport::SocketStream).
Decode is the exact inverse of encode, so `decode(encode(m)) == m`.
