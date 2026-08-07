One function of a `filter` list, as authored — in author order, and with a
`blur()` radius still UNRESOLVED.

Every filter function but `blur()` reaches the CSS lowering as numbers already
(the amounts are `<number>`/`<percentage>`, and `drop-shadow()`'s offsets ride
inside a [`BoxShadow`](crate::property::BoxShadow), which computed value folds).
`blur(<length>)` is the exception: `blur(1rem)` used to be flattened with
`Length::as_px`, which answers [`None`] for every relative unit — and the
lowering turned that [`None`] into a radius of `0`, so the element painted
sharp with nothing to grep for.

A list carrying one of these survives to computed-value time, where
[`absolutize_value`](crate::computed) resolves the radius against the node's
[`LengthContext`](crate::property::LengthContext) and rebuilds an ordinary
[`FilterOp`] list. Nothing downstream of computed value sees this type.
