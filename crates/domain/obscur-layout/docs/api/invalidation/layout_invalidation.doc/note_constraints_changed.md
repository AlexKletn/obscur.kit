**Downward gate.** Invalidates `ctx`'s node with
[`ConstraintsChanged`](LayoutInvalidationReason::ConstraintsChanged)
**iff** the context's size depends on the incoming constraints
(`ctx.should_remeasure(true, None)`), and returns whether it did.

This encodes §4.3's "propagate down only when constraints change": a
context whose measured size is independent of its constraints is not
invalidated by a constraint change.
