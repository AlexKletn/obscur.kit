Drops every binding and every piece of transition bookkeeping whose node is no
longer in the tree, and hands each dropped binding's timeline back to the
scheduler.

The tree is the liveness oracle, and a `NodeId` is generational: a remounted
node is a *different* id, so a dead entry is never rematched and never
self-heals. Nothing in this driver is keyed by anything else, so one
`RetainedTree::contains` per entry answers the whole question.

It costs one hash lookup per entry and no traversal, which is why it lives in
[`drive_css`](Self::drive_css) — a pass the style phase already runs with the
tree in hand — rather than in a walk of its own. Trading a leak for a
per-frame tree walk would only move the cost.

The timeline is removed with its binding. Left in the scheduler it would keep
`needs_frame` true for the life of the process, and the runtime would go on
rendering frames to advance an animation that no node can show.
