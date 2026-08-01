One declaration that applies to a node and competes in the cascade (§4.14).

A declaration pairs a target [`PropertyId`] with the [`PropertyValue`] to
assign, together with everything needed to rank it against the other
declarations that matched the same node: its `!important` flag, its
[`CascadeOrigin`], the [`SelectorSpecificity`] of the rule that produced it,
and its `source_order` position within that origin's stream.

The optional [`selector`](Declaration::selector) records which compiled
selector produced the declaration, purely as provenance for diagnostics; it
plays no part in ranking.

Because [`PropertyValue`] carries `f32` payloads it is not [`Eq`]; this type
therefore derives [`PartialEq`] but not [`Eq`].
