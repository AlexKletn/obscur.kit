Substitutes `var()` references, following chains with a visited guard so a
cycle terminates.

Returns a borrow of the referenced concrete value, or [`None`] when the
reference is unresolvable — the name is missing, or a cycle was detected —
which the caller spells [`PropertyValue::Unset`]. A non-reference value is
returned as a borrow of itself.

A borrow rather than a value because a custom property now carries the text it
was declared with (CSS Variables 1 §2), and the value being walked owns heap
where it used to own none: cloning each step of the chain bought a heap string
per `var()` reference, per node, per restyle, for a value that is read once and
dropped.
