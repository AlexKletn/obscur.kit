A node record in the retained tree, owned by the arena.

The retained tree is the single source of UI identity (spec §1.5): layout,
focus, selection, animation, DnD and caches all attach to a node's stable
[`NodeId`] plus its [`NodeGeneration`] (spec §2.2). Because the tree is an
arena, a node links to its parents and children **by `NodeId`**, never by
owning child nodes.

A node has an independently settable **logical** parent (structure /
style-theme scope) and **visual** parent (paint location). These coincide for
ordinary nodes but diverge for portalled nodes (spec §4.14).
Because the retained `props` carry a [`ViewValue`](obscur_view::view::ViewValue)
— which holds an `f64` — this type is `PartialEq` but not `Eq`.
