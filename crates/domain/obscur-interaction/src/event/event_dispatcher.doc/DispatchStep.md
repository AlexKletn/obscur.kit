One node visited in one phase along an event's propagation path (spec §4.7).

A [`DispatchStep`] pairs the [`node`](Self::node) reached with the
[`phase`](Self::phase) it is reached in, so an ordered list of steps fully
describes how an event threads the logical tree.
