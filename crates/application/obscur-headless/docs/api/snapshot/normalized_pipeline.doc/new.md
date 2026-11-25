Builds a normalized pipeline from its already-computed parts.

`nodes` is expected to be sorted by [`NodeId`];
[`SnapshotNormalizer`](super::SnapshotNormalizer) guarantees this. The
constructor stores the vector as given and does not re-sort.
