A value type in the .okui DSL's type system (spec §2.9).

This is the vocabulary of types a prop, state binding or expression result
can carry. It is deliberately small: the DSL types are the scalar
primitives, homogeneous [`List`](ValueType::List)s, [`Optional`](ValueType::Optional)
values (spec §2.10 null/none), and the opaque [`Component`](ValueType::Component)
type. Two escape hatches round it out: [`Unknown`](ValueType::Unknown) marks a
type that has not been inferred yet, and [`Any`](ValueType::Any) matches every
type.

The type *algebra* lives in [`ValueType::is_assignable_to`], which the typing
and lowering steps use to check that a value fits where it is used.
