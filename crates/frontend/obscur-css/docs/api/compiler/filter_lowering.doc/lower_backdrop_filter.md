Lowers a `backdrop-filter` value into a [`PropertyValue::Filter`] op list
carrying only `blur(<length>)` ops (the frosted-glass / acrylic effect).

`backdrop-filter` blurs what is painted BEHIND the element, not its own
content. Only the `blur(<length>)` function is modelled — it lowers to a
[`FilterOp::Blur`] the coordinator emits as a `BackdropFilter` command. A zero
radius (`blur(0)`, `blur()`) retains the operation for stacking and skips
the blur pass. Every other function (`brightness()`,
color-matrix, `drop-shadow()`, …) is **silently dropped** — out of scope for
the backdrop pass (documented). `backdrop-filter: none` and an empty value both
lower to an **empty** list (no effect). Returns [`None`] only when a `blur(…)`
is present but malformed (a negative or non-length radius), so the author sees
the mistake rather than a silent no-op.
