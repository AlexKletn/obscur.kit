A `transform` whose function list still carries unresolved lengths — the
SPECIFIED form of [`PropertyValue::Transform`].

The CSS lowering emits this only when it cannot flatten the list itself
(`translate(2em, 10vw)`, `translateY(calc(1rem + 4px))`);
[`absolutize_value`](crate::computed) folds it against the node's length context
and replaces it with a [`PropertyValue::Transform`]. It reaches a computed value
in this shape only when a distance is still unknown after that fold — today only
a `ch` with no measured advance — in which case the declaration is dropped
rather than painted at a guessed `0`.
