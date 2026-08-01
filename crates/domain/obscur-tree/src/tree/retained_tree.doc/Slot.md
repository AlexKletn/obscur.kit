One addressable slot in the [`RetainedTree`] arena.

A slot carries its own authoritative [`NodeGeneration`] independent of any
occupant, so that after a node is removed and the slot is later reused, ids
minted against the previous occupant can be recognised as stale. The
generation is advanced both when a slot is vacated and when it is reused.
