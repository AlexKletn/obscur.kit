Returns `true` if `one_based_index` is a term of `a*n + b` for some
integer `n >= 0`.

Examples: `2n+1` matches `1, 3, 5, …`; `0n+3` (i.e. `a == 0`) matches
only `3`; `-n+3` matches `1, 2, 3`.
