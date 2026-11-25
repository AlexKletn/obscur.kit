A value binding in the UI IR: either a literal or a dynamic reference
(spec §2.5).

Expressions are the serializable compile artifact a property's or event's
value lowers to. A [`Literal`](UiIrExpression::Literal) is a baked-in
constant; a [`Binding`](UiIrExpression::Binding) is a path into component
state or props resolved at runtime; a [`Call`](UiIrExpression::Call) applies
a named function or formatter to argument expressions.

Because [`UiIrValue`] contains an `f64`, this enum derives only `Debug`,
`Clone`, and `PartialEq`.
