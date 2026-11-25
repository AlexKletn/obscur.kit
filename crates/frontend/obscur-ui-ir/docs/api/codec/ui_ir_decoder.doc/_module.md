`UiIrDecoder`: the deterministic binary decoder for [`UiIrDocument`], and its
colocated [`UiIrDecodeError`] (spec §2.5 codecs).

The decoder is the exact inverse of
[`UiIrEncoder`](crate::codec::UiIrEncoder): it reads the little-endian layout
documented on that type and rebuilds an equal [`UiIrDocument`] through the
public constructors and builders. The round-trip
`decode(encode(doc)) == doc` is the contract.

Reading is driven by a private [`Cursor`] that tracks a position into the
byte slice; every read is bounds-checked and maps a short read to
[`UiIrDecodeError::UnexpectedEof`] and invalid UTF-8 to
[`UiIrDecodeError::InvalidUtf8`].

## Control-flow node kinds

The wire format reserves tags 4 (`Conditional`) and 5 (`Loop`), and the
encoder emits them faithfully, but [`UiIrNode`] currently exposes no public
constructor for those kinds. Until such constructors exist, decoding a node
with either tag yields [`UiIrDecodeError::UnsupportedNodeKind`] rather than
silently reinterpreting it. A faithful decode for these kinds awaits their
constructors.
