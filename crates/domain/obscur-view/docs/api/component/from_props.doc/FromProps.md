Builds a component's typed [`Props`](crate::component::Component::Props) from
the properties an authored node carries (spec §3.1: *values flow down*).

This is the decoding half of the boundary the component model was blocked on:
[`Component::render`](crate::component::Component::render) takes `&Self::Props`,
but a `.okui` document — or an `okui!` invocation, or the runtime's retained
tree — only has an untyped `&[ViewProperty]`. Nothing could call `render`
behind a `dyn` handle without a way to cross that gap.

Implement it with `#[derive(Contract)]` rather than by hand: the derive already
knows each field's name, type and optionality, and — importantly — resolves
each property's [`InternedSymbol`] through the same registry the view tree
uses, so the symbols actually match.

# Absence

A property that is not present, or is present as
[`Null`](ViewValue::Null) (which *is* absence of a value), is absent: an
`Option<T>` field becomes [`None`], and a required field is a
[`PropsError::Missing`].
