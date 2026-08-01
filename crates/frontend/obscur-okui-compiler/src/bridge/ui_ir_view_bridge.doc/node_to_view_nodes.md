Lowers one IR `node` into the desired-view node(s) it produces.

Most kinds produce exactly one [`ViewNode`]; a transparent
[`Fragment`](UiIrNodeKind::Fragment) and the statically materialized
[`Conditional`](UiIrNodeKind::Conditional)/[`Loop`](UiIrNodeKind::Loop)
splice their children, so the return is a `Vec`.
