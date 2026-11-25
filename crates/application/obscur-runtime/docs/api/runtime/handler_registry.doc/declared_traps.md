The scopes of the traps the last reconciliation saw, in document order.

A tree-declared trap has no open/close call to hang a lifetime on: it exists for
exactly as long as its element is committed. This is the previous answer, and the
difference between it and the current one is what says which traps opened and
which closed on this frame.
