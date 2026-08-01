Returns this value with its wrapped [`PropertyValue`] rewritten by `f`.

Crate-internal, because the concreteness invariant now rides on `f`
instead of on [`from_concrete`](Self::from_concrete)'s check. The only
caller is the computed-value absolutization pass, whose rewrite is
length-for-length — it can turn `2em` into `32px`, and cannot turn
anything into a CSS-wide keyword or a `var()` reference.
