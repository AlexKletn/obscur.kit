Inserts `node`, keyed by its own [`node`](SemanticNode::node) id, and
bumps the version (spec §7.1).

When `parent` is `Some`, the node's id is appended to that parent's
ordered child list. Re-inserting the same id **updates** the stored node
in place; the child list is only appended to when a `parent` is supplied,
so callers updating an already-linked node should pass `None` to avoid a
duplicate child entry.
