A stable, deterministically-ordered form of a
[`PipelineSnapshot`](super::PipelineSnapshot), for reproducible golden diffs
(spec §7.14).

A raw [`PipelineSnapshot`](super::PipelineSnapshot) stores its per-node stage
data in [`HashMap`](std::collections::HashMap)s whose iteration order is
unstable, so it is not directly suitable as golden output. A
`NormalizedPipeline` flattens that data into a `NodeId`-**sorted**
[`Vec<NormalizedNode>`] plus a few scalar summaries — one per §7.4 stage.
Because the ordering is total and deterministic, two normalizations of the
same frame produce byte-for-byte-equal output — the property golden tests
rely on (§7.14).

Produce one with [`SnapshotNormalizer`](super::SnapshotNormalizer).
