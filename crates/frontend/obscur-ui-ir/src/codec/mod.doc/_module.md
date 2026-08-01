Deterministic binary codecs for [`UiIrDocument`](crate::document::UiIrDocument)
(spec §2.5): a hand-rolled little-endian [`UiIrEncoder`]/[`UiIrDecoder`]
pair with no serde or external-crate dependency.

The two halves are exact inverses — `decode(encode(doc)) == doc` is the
contract. The wire format (magic, header, node/expression/value layout) is
documented on [`UiIrEncoder`].
