One color stop in a gradient: a [`Color`] positioned at a normalized
`offset` along the gradient's axis (spec §5.1).

The `offset` is conventionally in the `0.0..=1.0` range, where `0.0` is the
start of the gradient and `1.0` is its end. Values outside that range are
permitted but are not meaningful without an
[`ExtendMode`](super::ExtendMode) to interpret them.

Stops are **ordered by offset**: gradients store a `Vec<GradientStop>` that
callers are expected to supply pre-sorted in ascending `offset` order. This
type does not sort or validate; it is a plain value.
