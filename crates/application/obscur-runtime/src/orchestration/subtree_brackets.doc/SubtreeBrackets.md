The `Push*` / `Pop*` pairs bracketing each node's whole subtree paint, recorded
during the compose walk and turned into paint units once the walk knows where
every subtree ends.

A bracket is **one fact with two ends**. A `PushClip` is meaningless without the
`PopClip` that closes it, the pop cannot be emitted until the walk has seen the
subtree's last descendant, and the *relative* nesting of the six bracket kinds —
blur outside the transform, the transform outside the group layers, the group
layers outside the rounded clip, the scroll transform inside it — is a single
decision that has to be taken in one place or the display list stops being
balanced. This type owns all three: a `record_*` call takes both ends of one
bracket at once, and [`emit`](Self::emit) is the only code that decides how they
nest.

It therefore also owns the pre-order bookkeeping the pops are keyed by — each
node's enter index, its parent, its depth and the pre-order sequence — because
a subtree's **exit** index (the largest enter index within it) is derived from
exactly those and is needed by every kind of pop. Recording a node with
[`enter`](Self::enter) is what makes its brackets closable.

A walk that records nothing emits nothing, so a tree with no radius, overflow,
transform, opacity, blend, filter, outline or scroll produces a byte-for-byte
identical display list to one composed before brackets existed.
