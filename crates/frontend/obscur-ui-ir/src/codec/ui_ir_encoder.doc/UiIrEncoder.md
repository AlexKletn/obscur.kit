A stateless, deterministic binary encoder for [`UiIrDocument`] (spec §2.5).

The encoder serializes a document into the compact little-endian layout
documented at the [module level](self). It holds no state — every call is
independent — and encoding is total (it never fails). Decode the produced
bytes with [`UiIrDecoder`](crate::codec::UiIrDecoder).
