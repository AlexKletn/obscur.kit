A `filter` / `backdrop-filter` whose list still carries an unresolved `blur()`
radius — the SPECIFIED form of [`PropertyValue::Filter`].

The CSS lowering emits this only for a `blur(<relative length>)`;
[`absolutize_value`](crate::computed) resolves the radius against the node's
length context and replaces it with a [`PropertyValue::Filter`]. It reaches a
computed value in this shape only when the radius is still unknown after that
fold — today only a `ch` with no measured advance — in which case the
declaration is dropped rather than painted at a guessed `0`.
