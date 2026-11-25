Wraps a [`PropertyValue`] as a computed value, checking the concreteness
invariant.

Returns [`Some`] when `value` is a concrete leaf (keyword / length /
number / color) and [`None`] when it is a CSS-wide keyword
(`inherit` / `initial` / `unset`) or an unresolved
[`PropertyValue::CustomRef`] — none of which may appear in a computed
value.
