The already-cascaded style inputs the [`StyleCoordinator`] threads into the
resolver, keyed by [`NodeId`].

# Phase-1 stand-in

Matching a stylesheet against the tree and running the cascade is the job of
the `obscur-css` crate, which does not exist yet. Until it does, Phase 1
supplies the **already-cascaded** per-node values directly: a
[`ResolvedDeclarations`] (the winning cascaded value per property) plus the
node's cascaded custom properties. The [`StyleCoordinator`] never runs a
cascade itself — it only walks the tree and hands these inputs to
`obscur-style` (spec §3.10).

An unset node resolves to its default/inherited computed style: the getters
return shared empty inputs by reference when a node has no entry, so every
node in the tree can be resolved whether or not it was given inputs.

[`StyleCoordinator`]: super::StyleCoordinator
