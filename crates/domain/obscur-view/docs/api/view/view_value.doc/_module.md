`ViewValue`: a minimal, **provisional** value model for Phase 1.

# Provisional — will be superseded

This is a deliberately tiny, closed value enum that exists only so the
Phase 1 vertical slice can flow concrete property values through the desired
view tree. It is **not** the real value system: the eventual typed value
model (with its own schema, coercion, and identity rules) will replace this
type wholesale. Treat every use of `ViewValue` as a placeholder and avoid
building durable behaviour on its exact shape.

Because the `Float` variant carries an `f64`, this type derives only
`Debug`, `Clone`, and `PartialEq` — not `Eq`/`Hash` (floats are not totally
ordered and have `NaN`). Any type that transitively contains a `ViewValue`
inherits the same restriction.
