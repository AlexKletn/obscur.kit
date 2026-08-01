The synchronous, deterministic driver that runs one frame's core pipeline
(spec §3.4).

The pipeline is **orchestration glue only**: it owns a [`Reconciler`] and the
three stateless coordinators and calls them in a fixed order. It reimplements
no domain logic — reconciliation, the cascade, layout, and scene composition
each live entirely in their own crates.

Cross-frame state is *not* held here. The retained [`RetainedTree`] and the
mutable [`Scene`] are owned by the caller (the runtime) and passed into
[`run`](Self::run) by mutable reference, so the same pipeline value drives
every frame while the runtime keeps the evolving state.

# Phase-1 scope

[`run`](Self::run) executes the deterministic **core**: reconcile the desired
tree → resolve styles → layout → compose the scene → publish a snapshot. The
remaining §3.4 stages — input dispatch, motion sampling (pre- and
post-layout), render-graph construction, submission, and post-frame
notifications — are wired in later phases.
