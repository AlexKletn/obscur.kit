Appends the cardinality violation for one slot, if the bound proves one.

The number of children a slot ends up with is not a number but a **range**:
`certain` (the members that are definitely there) up to `max` (all of them,
or unbounded once a `<For>` body is among them). A violation exists only when
[`SlotCardinality::accepts`] rejects *every* count in that range — otherwise
the author has written something that may well be right, and a compile error
would be a lie.

The range is probed up to 2 because that is all a cardinality distinguishes:
none, one, more than one.
