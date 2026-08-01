A backend-neutral, GPU-free description of a frame's render graph (spec §7.4).

§7.4 lists the *render graph* as one of the eight headless outputs. A full
render graph — pass scheduling, transient targets, GPU resource aliasing — is
only meaningful with a concrete backend, so this type captures the
**backend-neutral structural description** that a headless frame determines:
the ordered [`passes`](Self::passes) and the [`target`](Self::target) they
render into. It invents no GPU data; every field is derived from the composed
scene and the viewport, so the same frame always yields the same summary.

For an ordinary headless frame the graph is a single `"scene"` pass (the
composited display list) into the viewport-sized target. Extension/custom
passes (spec §5.3) and GPU-specific passes are backend-gated and are not
modeled here; this is the deterministic skeleton every backend shares.
