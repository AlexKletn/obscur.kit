Types a reference to a resolved `symbol` as a dynamic value of its
`declared` type (spec §2.10).

A reference reads a value at runtime, so the result is
[`dynamic`](ExpressionType::dynamic) (never a compile-time constant). The
caller supplies `declared` from the prop/state type map; the `symbol` is
accepted so the signature is stable once per-symbol type lookup exists.
