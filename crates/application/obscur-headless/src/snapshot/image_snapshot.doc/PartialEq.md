Exact **content identity**: dimensions and content hash.

Retained pixels are deliberately *not* part of equality. Equality answers the
determinism question — same backend, same bytes — which the hash already
decides, and keeping it hash-based means a hash-only
[`from_parts`](ImageSnapshot::from_parts) snapshot still compares equal to the
raster it was digested from. The §7.14 tolerant comparison is
[`compare_within`](ImageSnapshot::compare_within), which is a different
question and is not an equivalence relation at all — it is not transitive.
