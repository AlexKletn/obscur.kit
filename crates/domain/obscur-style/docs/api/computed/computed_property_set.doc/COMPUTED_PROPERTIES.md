Every modelled longhand the resolver computes.

This is the complete [`PropertyId`] catalogue: the resolver must compute each
property that a downstream consumer reads back off a
[`ComputedStyle`](super::ComputedStyle)
(layout box-model / flex, scene paint, text typography), otherwise an authored
CSS value for that property would be silently dropped at the computed-value
step. It is spelled out as a local `const` list — rather than derived from the
enum — because [`PropertyId`] lives in another module this file must not edit;
the list is kept exhaustive by the `computed_properties_cover_every_property_id`
test, which fails if a new longhand is added upstream without being computed.
