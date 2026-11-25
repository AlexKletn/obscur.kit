Builds a [`SemanticTree`] from `tree`, marking `focused` (if live) as the
focused node.

Walks from the retained root, preserving child order and the retained
[`NodeId`] of every node. Returns an empty tree when the retained tree has
no designated root.
