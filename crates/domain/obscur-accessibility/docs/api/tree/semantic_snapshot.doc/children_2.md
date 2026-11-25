Returns the ordered child ids captured for `id`, or an empty slice if it
had none.

This is the captured tree's structure, and the source an adapter builds a
platform tree's hierarchy from. Ids are returned exactly as the live tree
held them, including any that name a node absent from this snapshot — a
caller that must produce a connected tree filters against
[`contains`](Self::contains).
