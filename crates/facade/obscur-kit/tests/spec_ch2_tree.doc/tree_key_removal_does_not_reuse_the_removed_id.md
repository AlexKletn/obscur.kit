§2.2: "nodes keep stable IDs where identity is preserved;"

FIXTURE REPAIRED 2026-07-19: same defect as
`tree_key_reorder_preserves_child_state` — `100 + i` made each row's
ComponentId shift when an earlier sibling was dropped, so "c" was component
102 in frame 1 and component 101 in frame 2 and could not possibly keep its
node. The ComponentId is now constant. The assertion is unchanged.
