The value of the property named `name`, or [`None`] when the node does not
carry it.

Properties are a short unordered slice, so this is a linear scan — the same
shape the reconciler already uses. The **first** match wins if a node somehow
carries a name twice.
