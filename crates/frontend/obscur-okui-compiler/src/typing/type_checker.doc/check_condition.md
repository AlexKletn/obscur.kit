Checks that `cond` may be used as a boolean condition (for example a
`<If cond={..}>`), reporting a
[`NotBoolean`](crate::typing::TypeErrorKind::NotBoolean) at `offset`
otherwise (spec §2.9).

# Errors

Returns a [`TypeError::not_boolean`] when `cond` is not assignable to
[`ValueType::Bool`].
