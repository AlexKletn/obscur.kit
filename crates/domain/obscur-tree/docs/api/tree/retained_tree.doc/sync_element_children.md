Recomputes `parent`'s element-only child projection
([`RetainedNode::children`]) from its all-nodes child list, keeping the
two in step. The arena is the only place that knows every child's
[`NodeKind`], so it — not a bare node — owns this derivation. Callers must
invoke it after any change to `parent.children` (link, unlink, reorder) or
after a child's kind changes. A no-op if `parent` is stale.

[`RetainedNode::children`]: crate::node::RetainedNode::children
