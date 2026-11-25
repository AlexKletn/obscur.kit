Emits a block expression evaluating to the `Vec<ViewNode>` that `nodes`
lower to, at `indent` levels.

A `Vec` built with `push` rather than a `vec![…]` literal, because a
conditional emits a real `if` — a statement, which no list literal can hold.
Each block declares its own sink, so a nested fragment's pushes cannot reach
an outer one.
