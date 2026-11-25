Positions one out-of-flow ([`Position::Absolute`] / [`Position::Fixed`])
child within its containing block and recurses into its subtree (spec §4.1).

The containing block is `parent_cb` (the nearest positioned ancestor's padding
box) for an absolute child and `viewport` for a fixed one. The child's natural
size is measured under a loose constraint bounded by the containing block
when either axis needs it, then
[`Inset::resolve_absolute_box`](obscur_layout::model::Inset::resolve_absolute_box) resolves its absolute border box from its insets
and explicit size. The subtree is arranged under a tight constraint at that box,
establishing the child's own padding box as the containing block for *its*
positioned descendants (the child is itself positioned).
