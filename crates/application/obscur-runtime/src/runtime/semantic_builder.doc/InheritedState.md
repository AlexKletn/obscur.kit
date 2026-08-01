The structural state a node's subtree inherits, carried down the walk.

The runtime's behavioural gates are **inherited, not per-node**:
`is_within_disabled` / `is_within_readonly` answer for the node's own
attribute *or any element ancestor's*, and a field counts as editable when
anything in its ancestor chain binds [`EVENT_INPUT`]. Reading only a node's
own attributes here would therefore still publish the opposite of what the
runtime does for a control nested inside a disabled panel, so the same
inheritance is folded down this walk.
