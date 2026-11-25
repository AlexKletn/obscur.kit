Returns the initial value of `property`, if it can be expressed without an
interner.

Value-typed properties have concrete initials: `color` is opaque black,
`background-color` is transparent, `opacity` is `1.0`, and `width` /
`height` are `auto`. Keyword-valued properties (`display`, `visibility`)
return [`None`]: their initial keyword is an interned symbol that Phase 1
cannot mint, so the property is left unresolved instead (see the module
docs).
