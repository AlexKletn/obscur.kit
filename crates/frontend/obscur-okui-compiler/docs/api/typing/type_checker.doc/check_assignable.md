Checks that `value` is assignable to `target`, reporting a
[`Mismatch`](crate::typing::TypeErrorKind::Mismatch) at `offset`
otherwise (spec §2.9).

# Errors

Returns a [`TypeError::mismatch`] when `value` is not assignable to
`target`.
