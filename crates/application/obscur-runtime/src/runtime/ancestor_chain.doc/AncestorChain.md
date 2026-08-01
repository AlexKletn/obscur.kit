A node and its logical ancestors, **node first** and root last.

Four different questions in this runtime are answered by walking exactly this
sequence — which nodes are hovered, which node's binding claims a bubbling
event, whether an enclosing control is disabled or readonly, and what §2.6's
routing chain visits — and each of them used to open-code the same walk.

# Why it is a type and not a loop

The walk has a **bound** that is easy to omit and impossible to notice missing:
it follows [`logical_parent`](obscur_tree::node::RetainedNode::logical_parent)
links, which are data, so a malformed parent chain that closes a cycle turns
any of those four questions into a hang. Capping the walk at the node count is
the entity's whole invariant, and stating it once is the only way it holds for
every caller.

*Logical* parents, not paint parents: a portal moves where a node paints but
not which element it belongs to (spec §4.14), so hover, bubbling and routing
all follow the authored structure.
