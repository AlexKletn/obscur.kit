A registration follows the **name**, not the node id: when the element stops
carrying the `test-id` the handler is released, and when it comes back the name
resolves again and the same handler runs. Without that, a positional registration
would be a one-frame accident on any UI that rebuilds a subtree — or, worse, would
keep answering for an element that gave the name up.

Note what the reconciler actually does here: the row keeps its component identity,
so removing its `test-id` **patches the node in place** rather than replacing it.
The node is therefore still live, and a binding that only checked liveness would
have stayed attached to an element that no longer answers to the name.
