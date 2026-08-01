Packs a slot `index` and `generation` into the opaque `u64` of a [`NodeId`].

The index occupies the low 32 bits and the generation the high 32 bits. This
packing is an arena-private encoding: callers only ever see an opaque
[`NodeId`] and must not depend on its layout.
