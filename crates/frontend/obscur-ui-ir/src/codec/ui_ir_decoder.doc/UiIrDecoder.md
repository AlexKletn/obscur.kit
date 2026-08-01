A stateless binary decoder for [`UiIrDocument`] (spec §2.5).

The decoder reads the little-endian layout produced by
[`UiIrEncoder`](crate::codec::UiIrEncoder) and rebuilds an equal document via
the public constructors and builders. It holds no state — every call is
independent.
