The inferred type of an expression's result (spec §2.10).

Beyond the [`ValueType`], this records whether the result `is_constant` — a
compile-time-constant literal (for example `42` or `"hi"`) as opposed to a
dynamic value read from a prop or state binding. The lowering step uses that
flag to decide whether an expression can be folded to a constant.
