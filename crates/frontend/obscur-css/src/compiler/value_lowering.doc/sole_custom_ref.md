The [`custom_ref`] of a component list that is exactly one `var()` reference,
and [`None`] otherwise.

A declaration whose entire value is one reference cannot be checked against the
property's grammar here, so the property-specific lowerings short-circuit on
this rather than throwing the declaration away.
