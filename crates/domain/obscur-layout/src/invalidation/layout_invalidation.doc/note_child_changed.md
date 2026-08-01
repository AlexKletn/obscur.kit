Invalidates `ctx`'s node with
[`ChildSizeChanged`](LayoutInvalidationReason::ChildSizeChanged) **iff**
`child`'s size feeds this context's measure
(`ctx.should_remeasure(false, Some(child))`), and returns whether it did.

A context that does not depend on `child` is left untouched (§4.3:
recompute only what the dependency graph says is affected).
