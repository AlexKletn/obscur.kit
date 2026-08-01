A named property on an IR node, flowing a [`UiIrExpression`] value *down*
into the node (spec §2.5).

A property pairs a name with an expression. The expression may be a baked-in
literal or a dynamic binding/call; [`is_dynamic`](UiIrProperty::is_dynamic)
reports which.
