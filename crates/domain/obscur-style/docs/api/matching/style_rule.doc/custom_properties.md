The custom properties (`--name: value`) this rule declares. They are held
apart from [`block`](Self::block) because they are not longhands: nothing
computes them, they are keyed by name rather than by
[`PropertyId`](crate::property::PropertyId), and they participate in the
cascade one step earlier than every other declaration (§4.14) so that a
`var()` in any of them has an answer before computed values are resolved.
