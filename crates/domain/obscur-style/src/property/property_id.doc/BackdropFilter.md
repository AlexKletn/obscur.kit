The `backdrop-filter` property (a `Filter` op list carrying only the
`blur(<length>)` function; NOT inherited). Unlike [`Filter`](Self::Filter)
— which blurs the element's OWN content — `backdrop-filter` blurs what is
painted BEHIND the element (the backdrop), producing frosted-glass /
Fluent acrylic. Only `blur(<length>)` is modelled; it lowers to a
[`FilterOp::Blur`](crate::property::FilterOp) that the scene coordinator
emits as a `BackdropFilter` command before the element's background, and the
Vello renderer frosts the backdrop with a segmented gaussian pass.
Color-matrix / other functions are out of scope (dropped); `none` /
`blur(0)` lowers to an empty list (no effect).
