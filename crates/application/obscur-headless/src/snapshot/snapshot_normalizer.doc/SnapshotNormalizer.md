Normalizes a [`PipelineSnapshot`] into a stable, deterministically-ordered
[`NormalizedPipeline`] for reproducible golden diffs (spec §7.14).

A raw snapshot keeps its per-node stage data in
[`HashMap`](std::collections::HashMap)s whose iteration order is unstable, so
it cannot be compared byte-for-byte across runs. The normalizer resolves
this by collecting every node that appears in the computed styles **or** the
layout boxes and emitting them **sorted by [`NodeId`]**, producing output
that is identical across runs of the same frame.

The normalizer is a zero-sized, stateless value; construct it with
[`new`](Self::new) or [`Default`].
