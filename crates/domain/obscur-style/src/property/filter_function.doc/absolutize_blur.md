Folds the `blur()` radius as far as this length context reaches, keeping the
rest symbolic; an already-resolved op is returned unchanged (its own lengths —
`drop-shadow()`'s offsets — are folded by the computed-value pass that owns
[`FilterOp`]).
