End-to-end verification that **authored border CSS** reaches paint and emits
the border fills at the box's edges through the *real* live pipeline (spec CSS
Backgrounds and Borders §4).

Each test authors CSS text, compiles it through the real `obscur-css` frontend
(`source -> parse -> compile -> matchable Stylesheet`), and renders one frame
on a [`UiRuntime`] via [`render_now`](UiRuntime::render_now), which runs
`reconcile -> match -> cascade -> resolve (ComputedStyles) -> layout
(LayoutBoxes) -> compose (DisplayList)`. The assertions read the *actual*
[`DisplayList`] the pipeline published (`result.snapshot().display_list()`),
never a hand-built scene, so a regression anywhere from the CSS `border`
shorthand through the computed-value step to the scene coordinator fails the
run.
