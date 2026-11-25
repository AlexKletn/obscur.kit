Typed identifiers for known longhand properties.

[`PropertyId`] is a *typed* enum, not a string: the cascade and computed-value
machinery dispatch on the variant, never on a property name. Names are kept
only as stable `&'static str`s for diagnostics and `DevTools`.

The set of variants here is a deliberate Phase-1 starter set. The full CSS
longhand catalogue grows in this enum over time; adding a longhand means
adding a variant plus its rows in [`PropertyId::name`] and
[`PropertyId::default_inheritance`].

Two facts *about* a property that this catalogue does not state variant by
variant live beside it rather than in it: the logical→physical mapping
(`logical_property.rs`) and the paint-only classification (`paint_only.rs`).
Both are further `impl PropertyId` blocks, so a caller still sees one type
with one set of methods.
