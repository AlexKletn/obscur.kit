Appends a diagnostic for every prop attribute given a literal of the wrong
type — a `String` prop handed an integer, an `i64` prop handed a string
(spec §2.8, "typed props … compile-time validation").

One diagnostic per offending attribute, anchored at that attribute, so two
wrong props on one element are two pointed errors, not one merged one. A
prop the contract records no type for (or records as
[`PropTypeSurface::Other`]), an attribute whose value is not a literal
(`count={total}` — a binding whose type is unknown here), and every event
attribute are all left alone.
