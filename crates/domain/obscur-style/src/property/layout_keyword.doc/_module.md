Reserved interned-symbol handles for layout keyword values.

A handful of layout properties (`display`, `flex-direction`,
`justify-content`, `align-items`) take their values from a small, *closed*
set of CSS keywords. Those keywords are carried through the style system as
[`PropertyValue::Keyword`](crate::property::PropertyValue::Keyword) values —
i.e. as [`InternedSymbol`] identities, never as strings.

The problem is that the CSS compiler and the runtime layout bridge live in
different crates and would ordinarily intern the same keyword text into
*different* symbol tables, so their `InternedSymbol`s could never be compared
for identity. To make the keyword handles agree across crates without sharing
an interner, this module reserves a fixed block of symbol values — one per
keyword — that both sides use directly. The CSS compiler resolves a keyword
*string* to its reserved symbol via the `*_from_name` helpers here; the
layout bridge compares a computed `Keyword(sym)` against the `pub const`s
below. These constants are the canonical cross-crate handles for layout
keyword values.

The reserved values occupy the `0x0000_1xxx` namespace (extended into
`0x0000_2xxx` once the `1xxx` category nibbles were exhausted). This is distinct
from `obscur-view`'s element-attribute reservations (`CLASS`/`ID`/`TYPE` in
`0x0000_000x`) and from the component/primitive prop namespaces (e.g.
`0x00C1_xxxx`), so a reserved layout keyword can never collide with either.
