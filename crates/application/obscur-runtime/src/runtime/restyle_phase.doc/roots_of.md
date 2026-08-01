The restyle roots a root reconcile implies, or [`None`] when its effect cannot be
bounded.

The distinction that matters is **structural vs in-place**:

* `Update` changed a node's own text or props. Nothing about the sibling *count*
  moved, so the reach is that node's subtree plus the siblings that follow it —
  what `+` and `~` can see. The node itself is the root.
* `Insert`, `Move` and `Remove` all change how many children the parent has, and
  the structural pseudo-classes count from **both** ends: `:last-child`,
  `:nth-last-child`, `:only-child` and `:nth-last-of-type` re-match for siblings
  *before* the edit too, and the parent's own `:empty` can flip. Expanding from
  the edited node would silently miss all of those, so the root is the **parent**,
  whose subtree covers every sibling in both directions.

`None` — a full pass — is left for the cases with no parent to expand from: a
removal of a root, or a `Move` whose node has somehow left the tree.

An associated function: it reads a plan and a tree and touches none of the
styling tier, so a frame can ask it before a phase exists.
