Reads the custom-property name out of a `var()`'s argument list and mints a
[`PropertyValue::CustomRef`] for it, or [`None`] when no `--`-prefixed name is
there.

A fallback (`var(--a, 8px)`) is not modelled: the first name wins and the rest
of the argument list is ignored.
