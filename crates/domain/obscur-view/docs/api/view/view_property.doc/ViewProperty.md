A single property set on a node in the desired view tree.

Data flows *down* into a component as properties (spec §3.1). The property
name is an [`InternedSymbol`] rather than an owned string so the desired
tree stays keyed on stable interned handles (spec §2.15), and the value is a
[`ViewValue`] — a Phase 1 placeholder value (see [`ViewValue`]'s docs).

Because [`ViewValue`] contains an `f64`, this type derives only `Debug`,
`Clone`, and `PartialEq`.
