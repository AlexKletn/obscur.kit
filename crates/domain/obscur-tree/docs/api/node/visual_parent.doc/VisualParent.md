A node's **visual** parent: where it is actually painted, i.e. its place in
the visual / stacking tree.

For an ordinary node the visual parent coincides with its
[`super::LogicalParent`]. For a portalled node they differ: the portal moves
the node's paint location (visual parent) while its logical parent — and thus
its style/theme scope — is preserved (spec §4.14). A newtype over
`Option<NodeId>`: `None` denotes a visual root.
