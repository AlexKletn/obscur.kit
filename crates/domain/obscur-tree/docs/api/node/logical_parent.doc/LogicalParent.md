A node's **logical** parent: its place in the component / structural tree,
which also defines its style and theme scope.

The logical parent is preserved across portals: a portal changes only where
a node is *painted* (its [`super::VisualParent`]), while its logical parent —
and therefore its style/theme scope — stays put (spec §4.14). A newtype over
`Option<NodeId>`: `None` denotes the tree root.
