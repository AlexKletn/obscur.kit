Per-node inverses of the accumulated CSS `transform` chain, for the nodes
that have one.

A node is absent from this map when neither it nor any logical ancestor
transforms — the overwhelmingly common case, which therefore costs one
failed hash lookup and no arithmetic on the pointer path. A node whose
accumulated transform is **singular** (a `scale(0)` somewhere in the
chain) is absent from the index entirely instead: it paints to nothing,
so it can be hit by nothing.

Storing the inverse rather than the forward matrix is deliberate: the
pointer path only ever maps screen → local, and inverting once per
laid-out frame is cheaper than inverting once per pointer event.
