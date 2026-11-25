The normalized, comparable pipeline data for a single node (spec §7.14).

A [`NormalizedNode`] is one row of a
[`NormalizedPipeline`](super::NormalizedPipeline): it pairs a node's
[`NodeId`] with the observable stage data keyed by that id — its computed
`opacity` (if the computed style carries one) and its absolute layout
`bounds` (if the node was arranged). Either may be absent, so both are
[`Option`]s.

The type is deliberately small and fully comparable ([`PartialEq`]): a
`NormalizedPipeline` holds these in a `NodeId`-sorted [`Vec`], so equal
frames produce equal, order-stable node lists — the basis for golden diffs.
