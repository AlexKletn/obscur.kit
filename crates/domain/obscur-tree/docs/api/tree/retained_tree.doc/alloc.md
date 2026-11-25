Allocates a fresh node of the given `kind`, minting a new [`NodeId`] and
returning it.

A vacated slot is reused when one is available (advancing that slot's
generation), otherwise a new slot is appended. The stored
[`RetainedNode`] is constructed with the freshly minted id. This is a
structural mutation and advances the tree [`version`](Self::version).

The arena addresses at most `u32::MAX` slots; this bound is far beyond any
realistic UI tree and cannot be reached in practice.
