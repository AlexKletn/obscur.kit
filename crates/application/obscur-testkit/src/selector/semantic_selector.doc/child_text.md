The text `node`'s only child holds, or `None` when it has no children.

The reading a component test does after every frame. A mounted component's
output *is* its host node's children, so this one call answers "what did the
component render this time" — and `None` answers the different question "did the
boundary take the subtree away", which is why the return is an `Option` rather
than a panic.

A non-text child is a panic, deliberately: every fixture that feeds it
renders a single text node, so a non-text child means the fixture changed under
the test rather than that the assertion should soften.

# Panics

If the child is not a text node. See above: that means the fixture changed, not
that the assertion is too strict.
