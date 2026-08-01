A node's CSS `outline` (focus ring) at its subtree *exit* index. The
outline paints OUTSIDE the node's rounded clip and ON TOP of the node's
whole subtree, so it sorts after every `Pop*` bracket at that index. Set to
[`i64::MAX`] — the ceiling — so it is `>=` every `*_pop_order` (which are
`i64::MAX - depth`); a tie with a depth-0 pop is broken by emitting the
outline bracket LAST (after all pops) so the stable sort keeps it after them.
