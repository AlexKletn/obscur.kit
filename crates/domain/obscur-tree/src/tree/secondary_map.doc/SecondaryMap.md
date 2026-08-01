A map from [`NodeId`] to a projection value `V`.

Heavy per-node projections — layout boxes, paint data, accessibility nodes,
backend handles — live in secondary maps keyed by the node's stable
[`NodeId`] (spec §1.5). A secondary map **augments** identity: the `NodeId`
remains owned and minted by the
[`RetainedTree`](super::RetainedTree), and this map merely attaches derived
data to it. It never becomes an alternate identity of its own.

This Phase-1 implementation is backed by a
[`HashMap`]. A dense generational-vector variant (indexing directly by slot,
like the arena) is a later optimization behind this same API.
