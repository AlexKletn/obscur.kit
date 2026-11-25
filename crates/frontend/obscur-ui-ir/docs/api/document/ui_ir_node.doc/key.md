Returns this node's stable key expression, if one is attached.

A [`Loop`](UiIrNodeKind::Loop) lowered from a `<For key={…}>` carries the
key here (spec §2.11); most other nodes have none.
