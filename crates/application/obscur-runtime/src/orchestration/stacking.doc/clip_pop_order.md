The phase for a subtree's closing `PopClip` at its *exit* index: sorts
after the node's paint and after every descendant (all at
[`PAINT_ORDER`](Self::PAINT_ORDER)), and — when several clips end at the
same index (a rightmost-leaf chain) — deeper clips close first
(descending `depth`), so nested clips unwind in the correct order.
