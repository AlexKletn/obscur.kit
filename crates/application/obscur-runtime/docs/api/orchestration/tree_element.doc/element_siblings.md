Returns the ordered element siblings this node participates in
(including itself), or a single-element list of just this node when the
node is top-level (its element parent is the synthetic root or absent).

Only [`Element`](obscur_tree::node::NodeKind::Element) children count as
siblings; text, synthetic, and root nodes are excluded, matching CSS's
"element sibling" semantics.
