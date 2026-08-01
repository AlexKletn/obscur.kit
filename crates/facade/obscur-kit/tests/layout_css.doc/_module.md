End-to-end verification that **authored layout CSS** reaches the layout
engine and produces the expected [`LayoutBox`](obscur_kit::layout::model::LayoutBox)
geometry through the *real* live pipeline (spec §4.1/§4.2/§4.6).

Every test drives the genuine path — author CSS text, compile it through the
real `obscur-css` frontend (`source -> parse -> compile -> matchable
Stylesheet`), and render one frame on a [`UiRuntime`] via
[`render_now`](UiRuntime::render_now), which runs `reconcile -> match ->
cascade -> resolve (ComputedStyles) -> layout (LayoutBoxes)`. The assertions
read the *actual* absolute [`LayoutBox`] coordinates the pipeline produced
(`runtime.layout_boxes()`), never a hand-built `ComputedStyle`/`LayoutStyle`,
so a regression in the resolver, the computed→layout bridge, or the flex/flow
algorithm fails the run.

The container is laid out under a fixed 300x300 viewport, so all numbers are
deterministic. A flex container that authors an explicit `width`/`height` is
sized to it, and its children, percentages, and `justify-content` all resolve
against that authored extent (verified below); a container with no explicit
size still fills the available viewport (the fill-available default).
