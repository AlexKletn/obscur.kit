Computes each node's **effective stacking z** for the whole tree.

Walks pre-order from the root (cycle-safe via a visited set); the root's
effective z is `0`. Each node's effective z is its own `z-index` when it is
positioned with a numeric `<integer>` `z-index` (see [`own_stacking_z`]),
otherwise it inherits its parent's effective z. This makes a positioned
subtree with a `z-index` stack together, and leaves every z at `0` when no
`z-index` is authored.
