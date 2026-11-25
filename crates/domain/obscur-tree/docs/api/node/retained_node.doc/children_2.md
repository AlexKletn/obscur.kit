Returns the element-only child links — the **component tree** a host
observes (spec §4.16). Text runs and synthetic parts are content, not
component-tree children, so they are excluded; a `.badge` holding only the
text "New" reports **zero** children here. Internal traversals that must
see every child node (layout, paint, hit-test, semantics, reconcile) use
[`child_nodes`](Self::child_nodes) instead.
