The node's logical parent, captured **before** the removal.

A removal changes the sibling count, so every remaining sibling's
backward-counting match (`:last-child`, `:nth-last-child`) and the
parent's own `:empty` can change. Without the parent there is nothing
left to reach them through — the node itself is already gone — so a
consumer had to widen to a full restyle. `None` only for a root.
