What a property computes to when nothing declares it: [`initial_value_of`].

One table, one fact per [`PropertyId`] — the CSS initial value, or [`None`]
where this phase cannot express it. It is a table rather than a match inside
the resolver because the question it answers is asked from three unrelated
places in the computed-value step (a property with no cascaded value, an
explicit `initial`, and an `unset` on a non-inherited property), and because
*which* properties it must leave unresolved is a fact about this crate's
Phase-1 limits rather than about resolution — see the resolver's module docs
for that argument.
