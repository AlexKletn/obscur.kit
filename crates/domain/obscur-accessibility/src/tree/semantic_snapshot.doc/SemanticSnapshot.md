An immutable, versioned snapshot of a [`SemanticTree`] (spec §7.1).

Platform adapters consume **snapshots**, not the live tree: a snapshot
clones the semantic nodes into an owned map, copies the tree's parent→child
links, captures the [`root`](Self::root), and stamps the tree's
[`version`](Self::version) at the moment of capture. It is immutable after
construction — it exposes only read accessors and no mutators.

Because the live tree advances its version on every mutation, a retained
snapshot can be checked with [`is_stale_against`](Self::is_stale_against)
against the tree's current version to decide whether it must be refreshed
(spec §7.1).
