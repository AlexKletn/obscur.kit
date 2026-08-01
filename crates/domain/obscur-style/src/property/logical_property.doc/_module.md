The physical property a **logical** one names, under a given `direction` and
`writing-mode` (spec §4.13, css-writing-modes-4 §6.2).

Twenty logical longhands, each resolving into one of the physical box-model
longhands through a mapping that rotates with the writing mode and flips with
the direction. It is written as [`PropertyId`]'s own `impl`, because the
answer is a fact about the variant — but it is not part of the enum's
*identity*, which is why this is the half that can leave the catalogue:
`PropertyId::name` and `PropertyId::default_inheritance` are stated variant by
variant in the enum's own doc comments, and nothing in the catalogue states
this mapping.

`PropertyId::is_logical` and [`PropertyId::LOGICAL`] are here for the same
reason: both are this one mapping asked a different way — membership in it,
and the list to scan when looking for the logical declarations that could feed
a given physical property.
