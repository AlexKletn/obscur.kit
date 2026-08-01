The definite (fixed/percentage) extent a single track contributes when fitting
auto-repeat copies. An `fr` / `auto` / intrinsic-keyword track has no definite
size — it cannot bound a repetition count — so it contributes `0` (a
documented fallback); a `minmax` uses its `min` bound as the definite floor.
