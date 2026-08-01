The ordered child id lists at the moment of capture, keyed by parent.

A snapshot is what every platform adapter builds its native tree from, and
§7.1's structure lives in these links, not in the nodes: without them an
adapter can only reconstruct hierarchy from explicit
[`Owns`](crate::semantic::SemanticRelation::Owns) relations, which a
projection of a plain parent→child tree has no reason to carry — so the
whole tree arrives flattened onto its root.
