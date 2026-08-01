Captures every backend-independent pipeline stage from the current state
of `tree`, deterministically (spec §5.13, §7.4, §7.14).

This re-runs the read-only style → layout → scene stages purely to
observe them; it takes `tree` by shared reference and **does not mutate
it**. Concretely it:

1. captures the retained tree via [`RetainedTree::snapshot`];
2. resolves [`ComputedStyles`] with a fresh [`StyleCoordinator`] against
   `inputs`;
3. derives [`LayoutBoxes`] with a fresh [`LayoutCoordinator`]
   ([`layout_styles_from`](LayoutCoordinator::layout_styles_from) then
   [`layout`](LayoutCoordinator::layout) under `viewport`);
4. composes a display list with a fresh [`SceneCoordinator`], installs it
   in a fresh [`Scene`], and snapshots it (the **scene** stage);
5. clones the composed display list as the **paint** stage;
6. projects the retained tree into a [`SemanticSnapshot`] via
   [`SemanticTreeBuilder`] (the **accessibility** stage, spec §7.1);
7. derives the backend-neutral [`RenderGraphSummary`] from the scene and
   `viewport` (the **render-graph** stage).

The rendered-**image** stage is backend-dependent and left absent (see
[`with_image`](Self::with_image)).

Because the coordinators are stateless and deterministic and the tree is
unchanged between calls, capturing the same frame twice yields equal stage
data. The call never panics.
