Returns `true` for the intrinsic sizing keywords — the ones that can only
be answered by measuring content.

[`resolve`](Self::resolve) returns [`None`] for these *and* for
[`Auto`](Self::Auto), which is deliberate (an algorithm with no intrinsic
support degrades to its `auto` behaviour rather than misbehaving) but makes
the two indistinguishable at that seam. This predicate is how an algorithm
that *does* support them tells them apart.
