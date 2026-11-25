An immutable, backend-independent capture of every frame-pipeline stage for
one frame, for golden testing (spec §7.4, §7.14).

A `PipelineSnapshot` bundles the eight stages §7.4 enumerates. Seven are
backend-independent and captured deterministically with no GPU:

1. the retained [`TreeSnapshot`];
2. the resolved [`ComputedStyles`];
3. the arranged [`LayoutBoxes`];
4. the composed [`SceneSnapshot`];
5. the paint-command stream ([`DisplayList`]);
6. the accessibility [`SemanticSnapshot`] (spec §7.1);
7. the backend-neutral [`RenderGraphSummary`].

The eighth stage, the rendered [`ImageSnapshot`], is *backend-dependent*: a
raster comes from a concrete renderer, not the backend-independent pipeline,
so [`capture`](Self::capture) leaves it absent and a caller attaches one with
[`with_image`](Self::with_image) after running a backend such as
`obscur-render-cpu`.

It is the concrete artefact behind the Phase-1 slice proof that
`component → desired tree → retained tree → style → layout → scene` runs
deterministically headless (§5.13).

# Order-independent equality

The type derives [`PartialEq`], and two of its fields are
[`HashMap`](std::collections::HashMap)s whose iteration order is unstable.
`HashMap` equality is order-independent, so two captures of the *same* frame
state compare equal even though their maps iterate in different orders. For a
stable, byte-for-byte comparable form (e.g. to write to a golden file), run
it through [`SnapshotNormalizer`](super::SnapshotNormalizer).
