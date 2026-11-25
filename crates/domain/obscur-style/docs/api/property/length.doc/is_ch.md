Returns `true` when this length is a `ch` term.

Every other unit resolves from numbers a caller already holds; `ch` alone
needs the `0` glyph *shaped* to answer. This predicate lets a caller find
out whether that measurement is worth taking before taking it — see
[`LengthContext::ch_base`](crate::property::LengthContext::ch_base).
