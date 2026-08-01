Substitutes `var()` references, following chains with a visited guard so a
cycle terminates.

Returns the referenced concrete value, or [`PropertyValue::Unset`] when the
reference is unresolvable — the name is missing, or a cycle was detected.
A non-reference value is returned unchanged.
