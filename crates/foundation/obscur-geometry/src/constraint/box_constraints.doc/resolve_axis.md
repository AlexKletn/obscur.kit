One axis of [`BoxConstraints::constrain_allowing_overflow`].

`min >= max` covers the tight case (`min == max`) and, defensively, an
over-constrained one — in both the bounds leave no choice to make, so the upper
bound is used rather than propagating a contradiction. `max > 0` keeps a zero
axis out of that branch: see the note on
[`constrain_allowing_overflow`](BoxConstraints::constrain_allowing_overflow).
