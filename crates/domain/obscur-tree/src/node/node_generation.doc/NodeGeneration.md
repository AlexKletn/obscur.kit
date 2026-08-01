A per-node generation counter.

The retained tree is an arena: when a slot's `NodeId` index is reused for a
new node, this generation is advanced so that references minted against the
previous occupant can be recognised as stale (spec §1.5). It wraps the shared
[`obscur_identity::Generation`] primitive so the whole workspace agrees on a
single generation representation.
