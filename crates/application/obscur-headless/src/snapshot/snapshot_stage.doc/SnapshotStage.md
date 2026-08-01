The pipeline stage a snapshot captures (spec §7.4).

Headless mode can output every stage of the frame pipeline for inspection
and golden testing (spec §7.4). §7.4 enumerates eight stages, and this enum
names all of them so a snapshot, a diff, or a diagnostic can refer to a stage
without string dispatch:

1. the reconciled retained tree;
2. the computed styles;
3. the layout boxes;
4. the composed scene;
5. the paint-command stream;
6. the accessibility (semantic) tree;
7. the render graph;
8. the rendered image.

# Ordering

The variants are declared in **pipeline order** — the exact order §7.4 lists
them — and the derived [`Ord`]/[`PartialOrd`] follow that declaration order.
Comparing two stages therefore tells you which one runs earlier in the frame.

# Backend dependence

The first seven stages are backend-independent and are captured
deterministically without a GPU (see
[`PipelineSnapshot`](super::PipelineSnapshot)). Only [`Image`](Self::Image) is
backend-dependent: a raster is produced by a concrete renderer (e.g. the
software `obscur-render-cpu` backend), so the headless pipeline models it as
an optional stage carried by
[`ImageSnapshot`](super::ImageSnapshot) rather than deriving it in
[`capture`](super::PipelineSnapshot::capture).
