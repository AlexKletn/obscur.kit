How `value` reads as **text content** — what a body `{binding}` renders.

A [`Text`](ViewValue::Text) is itself; a number or a boolean is its ordinary
`Display` form (`7`, `1.5`, `true`), which is the same rule the build-time
route follows, where the emitted Rust interpolates the value with
`Display`. [`Null`](ViewValue::Null) renders empty rather than the literal
word "null" — a bound-but-absent value is *nothing to show*, not a noun.
