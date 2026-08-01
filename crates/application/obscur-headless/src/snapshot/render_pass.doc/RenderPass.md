A single, backend-neutral render pass within a
[`RenderGraphSummary`](super::RenderGraphSummary) (spec §7.4).

A render pass is a structural, GPU-free description of *what* a backend would
draw for one stage of the frame: a stable [`label`](Self::label) and the
paint-command counts it covers. It fabricates no GPU data — the counts are
derived directly from the composed scene's display list, so the same frame
always yields the same pass.

The headless pipeline emits a single `"scene"` pass for an ordinary frame
(the composited display list). Extension/custom passes (spec §5.3) and
GPU-specific passes are backend-gated and are not modeled here; when a real
backend runs them, it may append its own passes to the graph.
