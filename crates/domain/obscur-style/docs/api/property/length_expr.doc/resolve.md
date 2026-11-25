Resolves this expression to concrete logical pixels against `ctx`.

Returns [`None`] when any term cannot be folded (an `auto` term, a `%`
with no known [`percent_basis`](crate::property::LengthContext), a
division by zero, or an empty `min()` / `max()` list).
