Why decoding a node's [`ViewProperty`](crate::view::ViewProperty) slice into a
component's typed [`Props`](crate::component::Component::Props) failed
(spec §3.1).

Decoding is the boundary where untyped, authored data becomes typed inputs, so
its failures are *data* errors, not bugs: a `.okui` author can write a missing
or wrongly-typed attribute, and the runtime must report that rather than panic.
Both variants name the property by its [`InternedSymbol`] — the same symbol the
view tree carries — so a caller can map it back to a name for a diagnostic.
