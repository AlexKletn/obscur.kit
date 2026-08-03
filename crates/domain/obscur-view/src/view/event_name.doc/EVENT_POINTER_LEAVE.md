The pointer's hover chain no longer contains this node —
`on:pointerleave="…"`.

The other half of
[`EVENT_POINTER_ENTER`](crate::view::EVENT_POINTER_ENTER), which states what
the pair means in this toolkit, why the binding does not bubble while the
command it names does, and why `prevent_default` has nothing to suppress on a
crossing.

Leaves are dispatched **before** enters and **innermost-first**: a row is left
before the container that held it, and the whole old chain has been unwound
before any of the new one is entered. An `on:pointerleave` that tears down what
its `on:pointerenter` built can therefore assume it runs before the next node's
entry, even when the pointer crosses from one sibling straight into another.

A node that left the tree between frames is not dispatched to — it is not there
to receive anything — so a chain emptied by a removal is silent rather than
addressed to a dead id.
