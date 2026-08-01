A small bitset of structural and behavioural properties of a retained node.

These describe *what the node is or does* in the tree, not invalidation
state: per-node dirty / needs-layout flags are deliberately **not** modelled
here, because invalidation is a dependency-graph concern handled in later
crates (spec §4.3).

The bitset is a plain `u32` newtype with associated `const` flag values; no
external bitflags crate is used.
