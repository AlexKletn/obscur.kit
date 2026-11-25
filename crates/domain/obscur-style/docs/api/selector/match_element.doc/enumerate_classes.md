Offers `visit` every class this element carries, and reports whether it was
able to.

[`has_class`](Self::has_class) answers "do you carry *this* one?", which is all
the selector engine ever needs. An index over the sheets needs the opposite
direction — "which of your classes name a bucket?" — and that question cannot
be built out of `has_class` without asking it once per class in the sheets,
which is the walk the index exists to avoid.

The default answers `false` and visits nothing: an element type that has not
been taught to list its classes is not assumed to have none. `false` obliges
the caller to fall back to whatever it would have done without an index, so
the default is slower but never wrong.

An implementation that returns `true` promises the visited set agrees with
[`has_class`](Self::has_class): every class it visits answers `true` there, and
every class it does not visit answers `false`. Disagreement is how rules go
missing from a cascade.
