CSS custom properties (`--name: value`).

A custom property pairs its `--`-prefixed name (interned to an
[`InternedSymbol`]) with the value it was declared with — which, unlike every
other declaration in this crate, is not a typed value but the token sequence
the author wrote (see [`CustomProperty`]). Per the Obscur Kit 2 draft (§4.14),
custom properties **always inherit by default** and participate in cascade and
inheritance before computed-value resolution. Because that inheritance
behaviour is fixed, it is documented here rather than stored per instance.
