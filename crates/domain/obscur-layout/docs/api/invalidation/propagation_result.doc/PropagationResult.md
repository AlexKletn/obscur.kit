The outcome of propagating a layout change **upward** from a context toward
its parent (spec §4.3).

§4.3 propagates invalidation upward *only when a context's output geometry
changes*: an unchanged result cannot affect ancestors and stops there.
Independently, a **containment boundary** halts upward propagation even when
geometry changed, because contained layout cannot escape the boundary to
affect ancestors.

This value pairs those two facts so a caller can decide, via
[`continues_upward`](Self::continues_upward), whether to keep walking toward
the parent.
