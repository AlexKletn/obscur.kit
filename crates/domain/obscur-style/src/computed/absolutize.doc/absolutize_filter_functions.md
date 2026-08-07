Resolves a pending `blur()` radius and rebuilds an ordinary
[`FilterOp`](crate::property::FilterOp) list.

This is where `filter: blur(1rem)` gets its radius. The CSS lowering used to ask
`Length::as_px` for it, which answers [`None`] for every relative unit, and then
took `0` for the answer — a backdrop that rendered sharp with nothing to grep
for. The radius now travels as a [`LengthExpr`](crate::property::LengthExpr) and
is folded here.

An unresolvable radius leaves the whole list symbolic rather than blurring by
`0`: a filter chain is ordered, and dropping one op silently changes what the
rest of the chain sees.
