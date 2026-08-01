Returns the interaction [`PseudoState`] bit this pseudo-class requires,
or [`None`] for structural pseudo-classes (and for the negation-based
[`Enabled`](Self::Enabled), which is evaluated as "not disabled" by the
matcher rather than by a single required bit).
