The pending-restyle accumulator: [`StyleInvalidation`] (§4.3, §4.14).

[`StyleInvalidation`] is a small map of `NodeId -> StyleInvalidationReason`
recording which nodes must be restyled and why. The runtime feeds it changes
and consults each node's [`StyleDependency`], so the invalidation engine is
decoupled from the tree: this type never walks the tree itself, it only
records and merges the resulting invalidations (§4.3). Confining recalculation
to the recorded nodes is how invalidation stays limited to affected consumers
(§4.14).
