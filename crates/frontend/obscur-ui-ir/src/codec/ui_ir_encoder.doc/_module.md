`UiIrEncoder`: the deterministic binary encoder for [`UiIrDocument`]
(spec §2.5 codecs).

# Binary format (version 1)

All multi-byte integers and floats are **little-endian**, written via
`to_le_bytes`. The encoding is total: it never fails.

- **Header**: magic `b"OKIR"` (4 bytes), then the format version `u16` = 1.
- **Document**: the [`UiIrVersion`] (`major` u16, then `minor` u16); the
  `required_capabilities` as a `u32` count followed by each id as a
  length-prefixed string; then the root [`UiIrNode`].
- **Node**: `id` u64; `kind` u8 (Element=0, Text=1, Component=2, Fragment=3,
  Conditional=4, Loop=5); `tag` string; `content` string; `properties`
  (`u32` count, each = name string + expression); `events` (`u32` count,
  each = event string + handler string); `children` (`u32` count, each a
  node, recursive).
- **Expression**: a `u8` tag — 0 `Literal` (+ value), 1 `Binding`
  (+ string), 2 `Call` (+ name string + `u32` arg count + each arg an
  expression, recursive).
- **Value**: a `u8` tag — 0 `Null`, 1 `Bool` (+ u8), 2 `Int` (+ i64),
  3 `Float` (+ f64), 4 `Text` (+ string).
- **String**: a `u32` byte-length then the UTF-8 bytes.

The layout is fixed and field order is stable, so encoding is deterministic:
`encode(doc)` is byte-identical across runs. Decode it with
[`UiIrDecoder`](crate::codec::UiIrDecoder); the round-trip
`decode(encode(doc)) == doc` is the contract.
