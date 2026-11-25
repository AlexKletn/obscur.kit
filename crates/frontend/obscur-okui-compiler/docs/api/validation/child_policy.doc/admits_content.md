Returns whether non-element **content** — a text child or an `{expr}`
interpolation — is admitted.

Only [`None`](Self::None) refuses it, and for a reason that is not about
names: it says the position takes no children at all, so text written
there renders nowhere. [`Only`](Self::Only) is a list of *tags*, and a text
run has none to be absent from — restricting content on the strength of a
tag list would be a rule the list does not state.
