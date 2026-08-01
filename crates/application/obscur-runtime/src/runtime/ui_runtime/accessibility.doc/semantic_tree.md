Builds the semantic accessibility tree from the live retained tree
(spec §7.1).

This is the missing link §7.1 mandates — "accessibility uses a semantic
tree linked to the retained UI tree". Each retained node becomes a
[`SemanticNode`](obscur_accessibility::semantic::SemanticNode) keyed by the
same [`NodeId`], with its role/name/state/actions derived from the node's
kind, CSS type/local name, text content, flags, structural state
attributes and event bindings; the runtime's current
[`focused`](Self::focused) node is reflected as the focused semantic node.
The retained tree is left untouched.

The published state agrees with this runtime's own interaction gates: a
control that `StructuralStates` reports within a disabled one — itself or
via an ancestor — takes no focus and fires no command here, and is
announced disabled and non-actionable there. See
[`SemanticTreeBuilder`] for what the retained tree cannot answer
(notably [`bounds`](obscur_accessibility::semantic::SemanticNode::bounds),
which is layout output this projection never sees).

[`NodeId`]: obscur_identity::NodeId
